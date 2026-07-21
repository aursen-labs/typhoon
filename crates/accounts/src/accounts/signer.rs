use {
    crate::{AccountData, FromRaw, SignerAccount, UncheckedAccount, ValidateView},
    core::{marker::PhantomData, ops::Deref},
    solana_account_view::AccountView,
    typhoon_errors::{Error, ErrorCode},
};

pub type UncheckedSigner<'a, T> = Signer<'a, T, NoCheck>;

pub trait SignerCheck {
    fn check(_info: &AccountView) -> Result<(), Error> {
        Ok(())
    }
}

pub struct Check;

impl SignerCheck for Check {
    fn check(info: &AccountView) -> Result<(), Error> {
        if info.is_signer() {
            Ok(())
        } else {
            Err(ErrorCode::AccountNotSigner.into())
        }
    }
}

pub struct NoCheck;

impl SignerCheck for NoCheck {}

pub struct Signer<'a, T = UncheckedAccount<'a>, C = Check>
where
    T: AsRef<AccountView>,
    C: SignerCheck,
{
    pub(crate) acc: T,
    _phantom: PhantomData<&'a C>,
}

impl<T, C> ValidateView for Signer<'_, T, C>
where
    T: AsRef<AccountView> + ValidateView,
    C: SignerCheck,
{
    #[inline(always)]
    fn validate(info: &AccountView) -> Result<(), Error> {
        C::check(info)?;
        T::validate(info)
    }
}

impl<'a, T, C> TryFrom<&'a AccountView> for Signer<'a, T, C>
where
    C: SignerCheck,
    T: AsRef<AccountView> + TryFrom<&'a AccountView, Error = Error>,
{
    type Error = Error;

    #[inline(always)]
    fn try_from(info: &'a AccountView) -> Result<Self, Self::Error> {
        C::check(info)?;

        Ok(Signer {
            acc: T::try_from(info)?,
            _phantom: PhantomData,
        })
    }
}

impl<'a, T, C> From<Signer<'a, T, C>> for &'a AccountView
where
    C: SignerCheck,
    T: AsRef<AccountView> + Into<&'a AccountView>,
{
    #[inline(always)]
    fn from(value: Signer<'a, T, C>) -> Self {
        value.acc.into()
    }
}

impl<T, C> AsRef<AccountView> for Signer<'_, T, C>
where
    C: SignerCheck,
    T: AsRef<AccountView>,
{
    #[inline(always)]
    fn as_ref(&self) -> &AccountView {
        self.acc.as_ref()
    }
}

impl<T, C> SignerAccount for Signer<'_, T, C>
where
    T: AsRef<AccountView>,
    C: SignerCheck,
{
}

impl<T, C> Deref for Signer<'_, T, C>
where
    C: SignerCheck,
    T: AsRef<AccountView>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.acc
    }
}

impl<T, C> AccountData for Signer<'_, T, C>
where
    C: SignerCheck,
    T: AccountData + AsRef<AccountView>,
{
    type Data = T::Data;
}

impl<'a, T, C> FromRaw<'a> for Signer<'a, T, C>
where
    T: AsRef<AccountView> + FromRaw<'a>,
    C: SignerCheck,
{
    fn from_raw(info: &'a AccountView) -> Self {
        Self {
            acc: T::from_raw(info),
            _phantom: PhantomData,
        }
    }
}
