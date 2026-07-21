use {
    core::{cmp::min, mem::MaybeUninit, ptr::with_exposed_provenance_mut},
    pinocchio::{AccountView, MAX_TX_ACCOUNTS},
    solana_account_view::{RuntimeAccount, MAX_PERMITTED_DATA_INCREASE},
};

/// The "static" size of an account in the input buffer.
///
/// This is the size of the account header plus the maximum permitted data
/// increase.
const STATIC_ACCOUNT_DATA: usize = size_of::<RuntimeAccount>() + MAX_PERMITTED_DATA_INCREASE;

/// Advance the input pointer in relation to a non-duplicated account.
///
/// The macro will add `STATIC_ACCOUNT_DATA` and the account length to
/// the input pointer and align its address using [`align_pointer`].
macro_rules! advance_input_with_account {
    ($input:ident, $account:expr) => {{
        $input = $input.add(STATIC_ACCOUNT_DATA);
        $input = $input.add((*$account).data_len as usize);
        $input = with_exposed_provenance_mut(($input.expose_provenance() + (8 - 1)) & !(8 - 1));
    }};
}

/// A macro to repeat a pattern to process an account `n` times, where `n` is
/// the number of `_` tokens in the input.
///
/// The main advantage of this macro is to inline the code to process `n`
/// accounts, which is useful to reduce the number of jumps required.  As a
/// result, it reduces the number of CUs required to process each account.
///
/// Note that this macro emits code to update both the `input` and `accounts`
/// pointers.
macro_rules! process_n_accounts {
    // Base case: no tokens left.
    ( () => ( $input:ident, $accounts:ident, $accounts_slice:ident ) ) => {};

    // Recursive case: one `_` token per repetition.
    ( ( _ $($rest:tt)* ) => ( $input:ident, $accounts:ident, $accounts_slice:ident ) ) => {
        process_n_accounts!(@process_account => ($input, $accounts, $accounts_slice));
        process_n_accounts!(($($rest)*) => ($input, $accounts, $accounts_slice));
    };

    // Process one account.
    ( @process_account => ( $input:ident, $accounts:ident, $accounts_slice:ident ) ) => {
        // Increment the `accounts` pointer to the next account.
        $accounts = $accounts.add(1);

        // Read the next account.
        let account: *mut RuntimeAccount = $input as *mut RuntimeAccount;
        // Adds an 8-bytes offset for:
        //   - rent epoch in case of a non-duplicated account
        //   - duplicated marker + 7 bytes of padding in case of a duplicated account
        $input = $input.add(size_of::<u64>());

        if (*account).borrow_state != 255 {
            clone_account_view($accounts, $accounts_slice, (*account).borrow_state);
        } else {
            #[cfg(feature = "account-resize")]
            {
                // Stores the data length in the `padding` field. This is needed
                // to handle account resizing.
                (*account).padding = u32::to_le_bytes((*account).data_len as u32);
            }
            $accounts.write(AccountView::new_unchecked(account));
            advance_input_with_account!($input, account);
        }
    };
}

/// Create an [`AccountView`] referencing the same account referenced by the
/// [`AccountView`] at the specified `index`.
///
/// # Safety
///
/// The caller must ensure that:
///   - `accounts` pointer must point to an array of [`AccountView`]s where the
///     new [`AccountView`] will be written.
///   - `accounts_slice` pointer must point to a slice of [`AccountView`]s
///     already initialized.
///   - `index` is a valid index in the `accounts_slice`.
//
// Note: The function is marked as `cold` to stop the compiler from optimizing the parsing of
// duplicated accounts, which leads to an overall increase in CU consumption.
#[allow(clippy::clone_on_copy)]
#[cold]
#[inline(always)]
unsafe fn clone_account_view(
    accounts: *mut AccountView,
    accounts_slice: *const AccountView,
    index: u8,
) {
    accounts.write((*accounts_slice.add(index as usize)).clone());
}

/// Convenience macro to transform the number of accounts to process into a
/// pattern of `_` tokens for the [`process_n_accounts`] macro.
macro_rules! process_accounts {
    ( 1 => ( $input:ident, $accounts:ident, $accounts_slice:ident ) ) => {
        process_n_accounts!( (_) => ( $input, $accounts, $accounts_slice ));
    };
    ( 2 => ( $input:ident, $accounts:ident, $accounts_slice:ident ) ) => {
        process_n_accounts!( (_ _) => ( $input, $accounts, $accounts_slice ));
    };
    ( 3 => ( $input:ident, $accounts:ident, $accounts_slice:ident ) ) => {
        process_n_accounts!( (_ _ _) => ( $input, $accounts, $accounts_slice ));
    };
    ( 4 => ( $input:ident, $accounts:ident, $accounts_slice:ident ) ) => {
        process_n_accounts!( (_ _ _ _) => ( $input, $accounts, $accounts_slice ));
    };
    ( 5 => ( $input:ident, $accounts:ident, $accounts_slice:ident ) ) => {
        process_n_accounts!( (_ _ _ _ _) => ( $input, $accounts, $accounts_slice ));
    };
}

#[inline(always)]
#[allow(unused_assignments)]
pub unsafe fn deserialize<const MAX_ACCOUNTS: usize>(
    mut input: *mut u8,
    accounts: &mut [MaybeUninit<AccountView>; MAX_ACCOUNTS],
) -> usize {
    const {
        assert!(MAX_ACCOUNTS > 0, "MAX_ACCOUNTS must be at least 1");
        assert!(
            MAX_ACCOUNTS <= MAX_TX_ACCOUNTS,
            "MAX_ACCOUNTS must be less than or equal to MAX_TX_ACCOUNTS"
        );
    }

    let total = *(input as *const u64) as usize;
    input = input.add(size_of::<u64>());

    // Clamp up front — accounts beyond MAX_ACCOUNTS are never touched.
    let processed = if MAX_ACCOUNTS < MAX_TX_ACCOUNTS {
        min(total, MAX_ACCOUNTS)
    } else {
        total
    };

    if processed > 0 {
        let mut accounts = accounts.as_mut_ptr() as *mut AccountView;
        // Represents the beginning of the accounts slice.
        let accounts_slice = accounts;

        // The first account is always non-duplicated, so process
        // it directly as such.
        let account: *mut RuntimeAccount = input as *mut RuntimeAccount;
        #[cfg(feature = "account-resize")]
        {
            // Stores the data length in the `padding` field. This is needed
            // to handle account resizing.
            (*account).padding = u32::to_le_bytes((*account).data_len as u32);
        }
        accounts.write(AccountView::new_unchecked(account));

        input = input.add(size_of::<u64>());
        advance_input_with_account!(input, account);

        if processed > 1 {
            let mut to_process_plus_one = processed;

            if to_process_plus_one == 2 {
                process_accounts!(1 => (input, accounts, accounts_slice));
            } else {
                while to_process_plus_one > 5 {
                    // Process 5 accounts at a time.
                    process_accounts!(5 => (input, accounts, accounts_slice));
                    to_process_plus_one -= 5;
                }

                match to_process_plus_one {
                    5 => {
                        process_accounts!(4 => (input, accounts, accounts_slice));
                    }
                    4 => {
                        process_accounts!(3 => (input, accounts, accounts_slice));
                    }
                    3 => {
                        process_accounts!(2 => (input, accounts, accounts_slice));
                    }
                    2 => {
                        process_accounts!(1 => (input, accounts, accounts_slice));
                    }
                    1 => (),
                    _ => {
                        // SAFETY: `while` loop above makes sure that
                        // `to_process_plus_one` has 1 to 5 entries left.
                        unsafe { core::hint::unreachable_unchecked() }
                    }
                }
            }
        }
    }

    processed
}
