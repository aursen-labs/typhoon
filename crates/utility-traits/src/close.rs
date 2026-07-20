use {typhoon_accounts::WritableAccount, typhoon_errors::Error};

pub trait CloseAccount: WritableAccount {
    #[inline(always)]
    fn close(&mut self, destination: &mut impl WritableAccount) -> Result<(), Error> {
        let new_lamports = destination.lamports().wrapping_add(self.lamports());
        destination.set_lamports(new_lamports);
        self.set_lamports(0);

        // `AccountView::close` zeroes the owner, lamports, and data_len fields
        // in one shot (set owner = system program, lamports = 0, data_len = 0).
        self.as_mut().close().map_err(Into::into)
    }
}

impl<T: WritableAccount> CloseAccount for T {}
