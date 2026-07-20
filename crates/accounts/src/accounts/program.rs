use {
    crate::{ReadableAccount, ValidateView},
    core::marker::PhantomData,
    pinocchio::hint::unlikely,
    solana_account_view::AccountView,
    solana_program_error::ProgramError,
    typhoon_errors::Error,
    typhoon_traits::CheckProgramId,
};

///
/// Checks:
/// * `account_info.key == expected_program`
/// * `account_info.executable == true`
pub struct Program<'a, T> {
    info: &'a AccountView,
    _phantom: PhantomData<T>,
}

impl<T> ValidateView for Program<'_, T>
where
    T: CheckProgramId,
{
    #[inline]
    fn validate(info: &AccountView) -> Result<(), Error> {
        // Optimized program ID check using fast memory comparison
        if unlikely(!T::address_eq(info.address())) {
            return Err(ProgramError::IncorrectProgramId.into());
        }

        if !info.executable() {
            return Err(ProgramError::InvalidAccountOwner.into());
        }

        Ok(())
    }
}

impl<'a, T> TryFrom<&'a AccountView> for Program<'a, T>
where
    T: CheckProgramId,
{
    type Error = Error;

    #[inline]
    fn try_from(info: &'a AccountView) -> Result<Self, Self::Error> {
        <Program<'a, T> as ValidateView>::validate(info)?;
        Ok(Program {
            info,
            _phantom: PhantomData,
        })
    }
}

impl<'a, T> From<Program<'a, T>> for &'a AccountView {
    #[inline(always)]
    fn from(value: Program<'a, T>) -> Self {
        value.info
    }
}

impl<T> AsRef<AccountView> for Program<'_, T> {
    #[inline(always)]
    fn as_ref(&self) -> &AccountView {
        self.info
    }
}

impl<T> ReadableAccount for Program<'_, T> {}
