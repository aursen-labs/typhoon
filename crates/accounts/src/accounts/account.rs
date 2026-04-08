use {
    crate::{discriminator_matches, AccountData, FromRaw, ReadableAccount, System},
    core::marker::PhantomData,
    pinocchio::hint::unlikely,
    solana_account_view::AccountView,
    solana_program_error::ProgramError,
    typhoon_errors::{Error, ErrorCode},
    typhoon_traits::{CheckOwner, CheckProgramId, DataStrategy, Discriminator},
};

pub struct Account<'a, T>
where
    T: Discriminator,
{
    pub(crate) account: &'a AccountView,
    pub(crate) _phantom: PhantomData<T>,
}

impl<'a, T> TryFrom<&'a AccountView> for Account<'a, T>
where
    T: CheckOwner + Discriminator,
{
    type Error = Error;

    #[inline(always)]
    fn try_from(account: &'a AccountView) -> Result<Self, Self::Error> {
        // Check data length first - this is the cheapest check and most likely to fail
        if unlikely(account.data_len() < T::DISCRIMINATOR.len()) {
            return Err(ProgramError::AccountDataTooSmall.into());
        }

        // Validate discriminator using optimized comparison for small discriminators
        if unlikely(!discriminator_matches::<T>(account)) {
            return Err(ErrorCode::AccountDiscriminatorMismatch.into());
        }

        let owner = unsafe { account.owner() };

        // Verify account ownership - checked after discriminator for better branch prediction
        if unlikely(!T::owned_by(owner)) {
            return Err(ProgramError::InvalidAccountOwner.into());
        }

        // Handle special case: zero-lamport system accounts (least common case)
        if unlikely(System::address_eq(owner)) {
            // Only perform additional lamports check for system accounts
            if account.lamports() == 0 {
                return Err(ProgramError::UninitializedAccount.into());
            }
        }

        Ok(Account {
            account,
            _phantom: PhantomData,
        })
    }
}

impl<'a, T> From<Account<'a, T>> for &'a AccountView
where
    T: Discriminator,
{
    #[inline(always)]
    fn from(value: Account<'a, T>) -> Self {
        value.account
    }
}

impl<T> AsRef<AccountView> for Account<'_, T>
where
    T: Discriminator,
{
    #[inline(always)]
    fn as_ref(&self) -> &AccountView {
        self.account
    }
}

impl<T> ReadableAccount for Account<'_, T> where T: Discriminator {}

impl<T> AccountData for Account<'_, T>
where
    T: Discriminator + DataStrategy,
{
    type Data = T;
}

impl<'a, T> FromRaw<'a> for Account<'a, T>
where
    T: Discriminator,
{
    fn from_raw(account: &'a AccountView) -> Self {
        Self {
            account,
            _phantom: PhantomData,
        }
    }
}
