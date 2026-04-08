#![no_std]

pub use {accounts::*, discriminator::*, programs::*};
use {
    solana_account_view::{AccountView, Ref, RefMut},
    solana_address::Address,
    solana_program_error::ProgramError,
    typhoon_errors::Error,
    typhoon_traits::{Accessor, DataStrategy, Discriminator, MutAccessor},
};

mod accounts;
mod discriminator;
mod programs;

pub trait ReadableAccount: AsRef<AccountView> {
    #[inline(always)]
    fn address(&self) -> &Address {
        self.as_ref().address()
    }

    #[inline(always)]
    fn owned_by(&self, owner: &Address) -> bool {
        self.as_ref().owned_by(owner)
    }

    #[inline(always)]
    fn lamports(&self) -> u64 {
        self.as_ref().lamports()
    }

    #[inline(always)]
    fn raw_data(&self) -> Result<Ref<'_, [u8]>, ProgramError> {
        self.as_ref().try_borrow()
    }
}

pub trait WritableAccount: ReadableAccount {
    #[inline(always)]
    fn assign(&self, new_owner: &Address) {
        unsafe {
            self.as_ref().assign(new_owner);
        }
    }

    #[inline(always)]
    fn resize(&self, new_len: usize) -> Result<(), Error> {
        self.as_ref().resize(new_len).map_err(Into::into)
    }

    #[inline(always)]
    fn set_lamports(&self, lamports: u64) {
        self.as_ref().set_lamports(lamports);
    }

    #[inline(always)]
    fn raw_mut_data(&self) -> Result<RefMut<'_, [u8]>, ProgramError> {
        self.as_ref().try_borrow_mut()
    }
}

pub trait SignerAccount: ReadableAccount {}

pub trait AccountData: ReadableAccount {
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

pub trait WritableAccountData: AccountData + WritableAccount {
    #[inline(always)]
    fn mut_data(&self) -> Result<RefMut<'_, Self::Data>, Error>
    where
        <Self::Data as DataStrategy>::Strategy:
            for<'a> MutAccessor<'a, Self::Data, Data = &'a mut Self::Data>,
    {
        RefMut::try_map(self.as_ref().try_borrow_mut()?, |data| {
            <<Self::Data as DataStrategy>::Strategy as MutAccessor<'_, Self::Data>>::access_mut(
                &mut data[Self::Data::DISCRIMINATOR.len()..],
            )
        })
        .map_err(|_| ProgramError::InvalidAccountData.into())
    }
}

impl<T> WritableAccountData for T where T: AccountData + WritableAccount {}

pub trait FromRaw<'a> {
    fn from_raw(info: &'a AccountView) -> Self;
}
