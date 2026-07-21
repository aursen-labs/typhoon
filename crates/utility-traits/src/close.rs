use {core::ops::DerefMut, pinocchio::AccountView, typhoon_errors::Error};

pub trait CloseAccount: DerefMut<Target = AccountView> {
    #[inline(always)]
    fn close(
        &mut self,
        destination: &mut impl DerefMut<Target = AccountView>,
    ) -> Result<(), Error> {
        let account: &mut AccountView = self;
        let destination: &mut AccountView = destination;

        let new_lamports = destination.lamports().wrapping_add(account.lamports());
        destination.set_lamports(new_lamports);
        account.close().map_err(Into::into)
    }
}

impl<T: DerefMut<Target = AccountView>> CloseAccount for T {}
