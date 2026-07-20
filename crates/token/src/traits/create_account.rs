use {
    crate::TokenAccount,
    pinocchio::{cpi::Signer as CpiSigner, sysvars::rent::Rent, AccountView, Address},
    pinocchio_associated_token_account::instructions::{Create, CreateIdempotent},
    pinocchio_token::{instructions::InitializeAccount3, ID as TOKEN_PROGRAM_ID},
    typhoon_accounts::{Account, Mut, ReadableAccount, Signer, SignerCheck, WritableAccount},
    typhoon_errors::Error,
    typhoon_utility::create_account_with_minimum_balance_signed,
};

pub trait SplCreateToken<'a, T>
where
    Self: Sized + Into<&'a mut AccountView>,
{
    fn create_token_account(
        self,
        rent: &Rent,
        payer: &impl WritableAccount,
        mint: &impl ReadableAccount,
        owner: &Address,
        seeds: Option<&[CpiSigner]>,
    ) -> Result<Mut<'a, T>, Error> {
        let info: &'a mut AccountView = self.into();
        create_account_with_minimum_balance_signed(
            info,
            TokenAccount::LEN,
            &TOKEN_PROGRAM_ID,
            payer.as_ref(),
            rent,
            seeds.unwrap_or_default(),
        )?;

        InitializeAccount3 {
            account: info,
            mint: mint.as_ref(),
            owner,
        }
        .invoke()?;

        Ok(Mut::from_raw_info(info))
    }

    fn create_associated_token_account(
        self,
        payer: &impl WritableAccount,
        mint: &impl ReadableAccount,
        owner: &impl ReadableAccount,
        system_program: &impl ReadableAccount,
        token_program: &impl ReadableAccount,
    ) -> Result<Mut<'a, T>, Error> {
        let info: &'a mut AccountView = self.into();
        Create {
            funding_account: payer.as_ref(),
            account: info,
            wallet: owner.as_ref(),
            mint: mint.as_ref(),
            system_program: system_program.as_ref(),
            token_program: token_program.as_ref(),
        }
        .invoke()?;

        Ok(Mut::from_raw_info(info))
    }

    fn create_idempotent_associated_token_account(
        self,
        payer: &impl WritableAccount,
        mint: &impl ReadableAccount,
        owner: &impl ReadableAccount,
        system_program: &impl ReadableAccount,
        token_program: &impl ReadableAccount,
    ) -> Result<Mut<'a, T>, Error> {
        let info: &'a mut AccountView = self.into();
        CreateIdempotent {
            funding_account: payer.as_ref(),
            account: info,
            wallet: owner.as_ref(),
            mint: mint.as_ref(),
            system_program: system_program.as_ref(),
            token_program: token_program.as_ref(),
        }
        .invoke()?;

        Ok(Mut::from_raw_info(info))
    }
}

impl<'a> SplCreateToken<'a, Account<'a, TokenAccount>> for &'a mut AccountView {}

impl<'a, C> SplCreateToken<'a, Signer<'a, Account<'a, TokenAccount>, C>> for &'a mut AccountView where
    C: SignerCheck
{
}
