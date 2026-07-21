use {
    crate::{ReadableAccount, ValidateView},
    solana_account_view::AccountView,
    typhoon_errors::Error,
};

pub struct UncheckedAccount<'a> {
    info: &'a AccountView,
}

impl ValidateView for UncheckedAccount<'_> {
    #[inline(always)]
    fn validate(_info: &AccountView) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a> TryFrom<&'a AccountView> for UncheckedAccount<'a> {
    type Error = Error;

    #[inline(always)]
    fn try_from(info: &'a AccountView) -> Result<Self, Self::Error> {
        Ok(UncheckedAccount { info })
    }
}

impl<'a> From<UncheckedAccount<'a>> for &'a AccountView {
    #[inline(always)]
    fn from(value: UncheckedAccount<'a>) -> Self {
        value.info
    }
}

impl AsRef<AccountView> for UncheckedAccount<'_> {
    #[inline(always)]
    fn as_ref(&self) -> &AccountView {
        self.info
    }
}

impl ReadableAccount for UncheckedAccount<'_> {}
