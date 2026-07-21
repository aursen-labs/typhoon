use {
    pinocchio::{AccountView, Address},
    pinocchio_system::instructions::{Allocate, Assign, Transfer},
    typhoon_accounts::{
        Mut, Signer as SignerAccount, SignerCheck, SystemAccount, UncheckedAccount, WritableAccount,
    },
    typhoon_errors::Error,
};

pub trait SystemCpi<'a>: WritableAccount + Into<&'a AccountView>
where
    Self: Sized,
{
    #[inline(always)]
    fn allocate(&self, new_space: u64) -> Result<(), Error> {
        Allocate {
            account: self.as_ref(),
            space: new_space,
        }
        .invoke()
        .map_err(Into::into)
    }

    #[inline(always)]
    fn assign(&self, owner: &Address) -> Result<(), Error> {
        Assign {
            account: self.as_ref(),
            owner,
        }
        .invoke()
        .map_err(Into::into)
    }

    #[inline(always)]
    fn transfer(&self, to: &impl WritableAccount, amount: u64) -> Result<(), Error> {
        Transfer {
            from: self.as_ref(),
            lamports: amount,
            to: to.as_ref(),
        }
        .invoke()
        .map_err(Into::into)
    }
}

impl<'a, C: SignerCheck> SystemCpi<'a> for Mut<'a, SignerAccount<'a, SystemAccount<'a>, C>> {}
impl<'a, C: SignerCheck> SystemCpi<'a> for Mut<'a, SignerAccount<'a, UncheckedAccount<'a>, C>> {}
