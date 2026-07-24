use {
    crate::HandlerContext, core::marker::PhantomData, pastey::paste,
    solana_account_view::AccountView, solana_address::Address, solana_program_error::ProgramError,
    typhoon_errors::Error,
};

pub trait FromInfos<'a>: Sized {
    fn from_infos(accounts: &'a mut [AccountView]) -> Result<(Self, &'a mut [AccountView]), Error>;
}

macro_rules! impl_from_infos {
    ($($t:ident),+) => {
        impl<'a, $($t),+> FromInfos<'a> for ($($t),+) where $($t: TryFrom<&'a AccountView, Error = Error>),+ {
            fn from_infos(accounts: &'a mut [AccountView]) -> Result<(Self, &'a mut [AccountView]), Error> {
                paste! {
                    let [$( [<acc_ $t:lower>], )+ rem @ ..] = accounts else {
                        return Err(Error::new(ProgramError::NotEnoughAccountKeys));
                    };
                }

                paste! {
                    $( let [<val_ $t:lower>] = $t::try_from(&* [<acc_ $t:lower>])?; )+

                    Ok((( $( [<val_ $t:lower>] ),+ ), rem))
                }
            }
        }
    };
}

impl_from_infos!(T1, T2);
impl_from_infos!(T1, T2, T3);
impl_from_infos!(T1, T2, T3, T4);
impl_from_infos!(T1, T2, T3, T4, T5);

impl<'a, T: TryFrom<&'a AccountView, Error = Error>> FromInfos<'a> for (T,) {
    fn from_infos(accounts: &'a mut [AccountView]) -> Result<(Self, &'a mut [AccountView]), Error> {
        let [acc, rem @ ..] = accounts else {
            return Err(Error::new(ProgramError::NotEnoughAccountKeys));
        };

        let acc = T::try_from(&*acc)?;

        Ok(((acc,), rem))
    }
}

/// An iterator over account infos, yielding tuples of type `T` that can be constructed from
/// the current slice of accounts. The iterator advances by consuming the accounts as each item is produced.
pub struct AccountIter<'a, T> {
    accounts: &'a mut [AccountView],
    _phantom: PhantomData<T>,
}

impl<'b, T> HandlerContext<'_, 'b, '_> for AccountIter<'b, T> {
    fn from_entrypoint(
        _program_id: &Address,
        accounts: &'b mut [AccountView],
        _instruction_data: &mut &[u8],
    ) -> Result<(Self, &'b mut [AccountView]), typhoon_errors::Error> {
        Ok((
            AccountIter {
                accounts,
                _phantom: PhantomData,
            },
            &mut [],
        ))
    }
}

impl<'a, T> Iterator for AccountIter<'a, T>
where
    T: FromInfos<'a>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = core::mem::take(&mut self.accounts);
        match T::from_infos(slice) {
            Ok((item, rest)) => {
                self.accounts = rest;
                Some(item)
            }
            Err(_) => None,
        }
    }
}

/// Mutable sibling of [`FromInfos`].
///
/// Constructs a tuple from a *mutable* slice of accounts, handing each element
/// an `&mut AccountView` instead of a shared `&AccountView`. This is what lets
/// writable wrappers such as `Mut<Account<T>>` — whose only `TryFrom` impl is
/// `TryFrom<&mut AccountView>` — participate in variadic iteration.
pub trait FromInfosMut<'a>: Sized {
    fn from_infos_mut(
        accounts: &'a mut [AccountView],
    ) -> Result<(Self, &'a mut [AccountView]), Error>;
}

macro_rules! impl_from_infos_mut {
    ($($t:ident),+) => {
        impl<'a, $($t),+> FromInfosMut<'a> for ($($t),+) where $($t: TryFrom<&'a mut AccountView, Error = Error>),+ {
            fn from_infos_mut(accounts: &'a mut [AccountView]) -> Result<(Self, &'a mut [AccountView]), Error> {
                paste! {
                    // Slice patterns split a `&'a mut [_]` into disjoint `&'a mut _`
                    // bindings, so each element can be consumed mutably in turn.
                    let [$( [<acc_ $t:lower>], )+ rem @ ..] = accounts else {
                        return Err(Error::new(ProgramError::NotEnoughAccountKeys));
                    };
                }

                paste! {
                    $( let [<val_ $t:lower>] = $t::try_from([<acc_ $t:lower>])?; )+

                    Ok((( $( [<val_ $t:lower>] ),+ ), rem))
                }
            }
        }
    };
}

impl_from_infos_mut!(T1, T2);
impl_from_infos_mut!(T1, T2, T3);
impl_from_infos_mut!(T1, T2, T3, T4);
impl_from_infos_mut!(T1, T2, T3, T4, T5);

impl<'a, T: TryFrom<&'a mut AccountView, Error = Error>> FromInfosMut<'a> for (T,) {
    fn from_infos_mut(
        accounts: &'a mut [AccountView],
    ) -> Result<(Self, &'a mut [AccountView]), Error> {
        let [acc, rem @ ..] = accounts else {
            return Err(Error::new(ProgramError::NotEnoughAccountKeys));
        };

        let acc = T::try_from(acc)?;

        Ok(((acc,), rem))
    }
}

/// Mutable sibling of [`AccountIter`].
///
/// Yields tuples of *writable* account wrappers (e.g. `Mut<Account<T>>`) built
/// from the transaction's remaining accounts, so a handler can mutate an
/// unbounded list of accounts — airdropping to N wallets, settling N orders,
/// closing N PDAs, etc. Each yielded element is owner/discriminator-checked
/// exactly like its read-only counterpart, and writability is enforced.
pub struct AccountIterMut<'a, T> {
    accounts: &'a mut [AccountView],
    _phantom: PhantomData<T>,
}

impl<'b, T> HandlerContext<'_, 'b, '_> for AccountIterMut<'b, T> {
    fn from_entrypoint(
        _program_id: &Address,
        accounts: &'b mut [AccountView],
        _instruction_data: &mut &[u8],
    ) -> Result<(Self, &'b mut [AccountView]), typhoon_errors::Error> {
        Ok((
            AccountIterMut {
                accounts,
                _phantom: PhantomData,
            },
            &mut [],
        ))
    }
}

impl<'a, T> Iterator for AccountIterMut<'a, T>
where
    T: FromInfosMut<'a>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = core::mem::take(&mut self.accounts);
        match T::from_infos_mut(slice) {
            Ok((item, rest)) => {
                self.accounts = rest;
                Some(item)
            }
            Err(_) => None,
        }
    }
}
