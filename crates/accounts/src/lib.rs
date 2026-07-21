#![no_std]

pub use {accounts::*, discriminator::*, programs::*};
use {
    solana_account_view::{AccountView, Ref, RefMut},
    solana_program_error::ProgramError,
    typhoon_errors::Error,
    typhoon_traits::{Accessor, DataStrategy, Discriminator, MutAccessor},
};

mod accounts;
mod discriminator;
mod programs;

/// Validation logic that can be run against a borrowed [`AccountView`].
///
/// Read-only account wrappers (`Account`, `Signer`, `Program`, `SystemAccount`,
/// `UncheckedAccount`) perform their checks while constructing a `&AccountView`
/// reference. `Mut<'a, T>` reuses that same logic on a shared reborrow of its
/// `&'a mut AccountView` slot, then keeps the mutable reference for itself.
pub trait ValidateView {
    fn validate(view: &AccountView) -> Result<(), Error>;
}

pub trait SignerAccount: AsRef<AccountView> {}

pub trait AccountData: AsRef<AccountView> {
    type Data: Discriminator + DataStrategy;
}

pub trait ReadableAccountData: AccountData {
    #[inline(always)]
    fn data(&self) -> Result<Ref<'_, Self::Data>, ProgramError>
    where
        <Self::Data as DataStrategy>::Strategy:
            for<'a> Accessor<'a, Self::Data, Data = &'a Self::Data>,
    {
        Ref::try_map(self.as_ref().try_borrow()?, |data| {
            <<Self::Data as DataStrategy>::Strategy as Accessor<'_, Self::Data>>::access(
                &data[Self::Data::DISCRIMINATOR.len()..],
            )
        })
        .map_err(|_| ProgramError::InvalidAccountData)
    }

    #[inline(always)]
    fn data_owned(&self) -> Result<Self::Data, ProgramError>
    where
        <Self::Data as DataStrategy>::Strategy: for<'a> Accessor<'a, Self::Data, Data = Self::Data>,
    {
        self.as_ref().check_borrow()?;
        let data = unsafe { self.as_ref().borrow_unchecked() };
        <<Self::Data as DataStrategy>::Strategy as Accessor<'_, Self::Data>>::access(
            &data[Self::Data::DISCRIMINATOR.len()..],
        )
    }

    #[inline(always)]
    fn data_unchecked(
        &self,
    ) -> Result<
        <<Self::Data as DataStrategy>::Strategy as Accessor<'_, Self::Data>>::Data,
        ProgramError,
    >
    where
        <Self::Data as DataStrategy>::Strategy: for<'a> Accessor<'a, Self::Data>,
    {
        let data = unsafe { self.as_ref().borrow_unchecked() };
        <<Self::Data as DataStrategy>::Strategy as Accessor<'_, Self::Data>>::access(
            &data[Self::Data::DISCRIMINATOR.len()..],
        )
    }
}

impl<T> ReadableAccountData for T where T: AccountData {}

pub trait WritableAccountData: AccountData + AsMut<AccountView> {
    #[inline(always)]
    fn mut_data(&mut self) -> Result<RefMut<'_, Self::Data>, Error>
    where
        <Self::Data as DataStrategy>::Strategy:
            for<'a> MutAccessor<'a, Self::Data, Data = &'a mut Self::Data>,
    {
        RefMut::try_map(self.as_mut().try_borrow_mut()?, |data| {
            <<Self::Data as DataStrategy>::Strategy as MutAccessor<'_, Self::Data>>::access_mut(
                &mut data[Self::Data::DISCRIMINATOR.len()..],
            )
        })
        .map_err(|_| ProgramError::InvalidAccountData.into())
    }
}

impl<T> WritableAccountData for T where T: AccountData + AsMut<AccountView> {}

pub trait FromRaw<'a> {
    fn from_raw(info: &'a AccountView) -> Self;
}
