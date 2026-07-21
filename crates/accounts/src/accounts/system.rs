use {
    crate::{ReadableAccount, System, ValidateView},
    pinocchio::hint::unlikely,
    solana_account_view::AccountView,
    solana_program_error::ProgramError,
    typhoon_errors::Error,
    typhoon_traits::CheckProgramId,
};

pub struct SystemAccount<'a> {
    info: &'a AccountView,
}

impl ValidateView for SystemAccount<'_> {
    #[inline(always)]
    fn validate(info: &AccountView) -> Result<(), Error> {
        if unlikely(!System::address_eq(info.owner())) {
            return Err(ProgramError::InvalidAccountOwner.into());
        }
        Ok(())
    }
}

impl<'a> TryFrom<&'a AccountView> for SystemAccount<'a> {
    type Error = Error;

    #[inline(always)]
    fn try_from(info: &'a AccountView) -> Result<Self, Self::Error> {
        <SystemAccount<'a> as ValidateView>::validate(info)?;
        Ok(SystemAccount { info })
    }
}

impl<'a> From<SystemAccount<'a>> for &'a AccountView {
    #[inline(always)]
    fn from(value: SystemAccount<'a>) -> Self {
        value.info
    }
}

impl AsRef<AccountView> for SystemAccount<'_> {
    #[inline(always)]
    fn as_ref(&self) -> &AccountView {
        self.info
    }
}

impl ReadableAccount for SystemAccount<'_> {}
