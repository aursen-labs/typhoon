use {
    core::ops::DerefMut,
    pinocchio::{error::ProgramError, AccountView},
    typhoon_accounts::{Mut, Signer, SignerAccount, SignerCheck, SystemAccount, UncheckedAccount},
    typhoon_errors::Error,
};

pub trait LamportsChecked: DerefMut<Target = AccountView> + SignerAccount {
    #[inline(always)]
    fn send(
        &mut self,
        to: &mut impl DerefMut<Target = AccountView>,
        amount: u64,
    ) -> Result<(), Error> {
        let payer_lamports = self.lamports();
        let recipient_lamports = to.lamports();

        self.set_lamports(
            payer_lamports
                .checked_sub(amount)
                .ok_or(ProgramError::ArithmeticOverflow)?,
        );
        to.set_lamports(recipient_lamports.wrapping_add(amount));

        Ok(())
    }

    #[inline(always)]
    fn send_all(&mut self, to: &mut impl DerefMut<Target = AccountView>) -> Result<(), Error> {
        let amount = self.lamports();
        let recipient_lamports = to.lamports();

        self.set_lamports(0);
        to.set_lamports(recipient_lamports.wrapping_add(amount));

        Ok(())
    }
}

impl<C: SignerCheck> LamportsChecked for Mut<'_, Signer<'_, SystemAccount<'_>, C>> {}
impl<C: SignerCheck> LamportsChecked for Mut<'_, Signer<'_, UncheckedAccount<'_>, C>> {}
