use {
    crate::{
        AccountData, FromRaw, ReadableAccount, Signer, SignerAccount, SignerCheck, WritableAccount,
    },
    solana_account_view::AccountView,
    typhoon_errors::Error,
};

pub struct Mut<T: ReadableAccount>(pub(crate) T);

impl<'a, T> TryFrom<&'a AccountView> for Mut<T>
where
    T: TryFrom<&'a AccountView, Error = Error> + ReadableAccount,
{
    type Error = Error;

    #[inline(always)]
    fn try_from(info: &'a AccountView) -> Result<Self, Self::Error> {
        Ok(Mut(T::try_from(info)?))
    }
}

impl<T> AsRef<AccountView> for Mut<T>
where
    T: ReadableAccount,
{
    #[inline(always)]
    fn as_ref(&self) -> &AccountView {
        self.0.as_ref()
    }
}

impl<'a, T> From<Mut<T>> for &'a AccountView
where
    T: ReadableAccount + Into<&'a AccountView>,
{
    #[inline(always)]
    fn from(value: Mut<T>) -> Self {
        value.0.into()
    }
}

impl<T> ReadableAccount for Mut<T> where T: ReadableAccount {}
impl<T> WritableAccount for Mut<T> where T: ReadableAccount {}

impl<T> AccountData for Mut<T>
where
    T: AccountData + ReadableAccount,
{
    type Data = T::Data;
}

impl<T, C> SignerAccount for Mut<Signer<'_, T, C>>
where
    T: ReadableAccount,
    C: SignerCheck,
{
}

#[doc(hidden)]
impl<'a, T> Mut<T>
where
    T: ReadableAccount + FromRaw<'a>,
{
    #[inline(always)]
    pub fn from_raw_info(info: &'a AccountView) -> Self {
        Mut(T::from_raw(info))
    }
}
