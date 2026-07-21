use {
    crate::TokenAccount,
    pinocchio::{cpi::Signer as CpiSigner, sysvars::rent::Rent, AccountView, Address},
    pinocchio_associated_token_account::instructions::{Create, CreateIdempotent},
    pinocchio_token::{instructions::InitializeAccount3, ID as TOKEN_PROGRAM_ID},
    typhoon_accounts::{Account, Mut, Signer, SignerCheck},
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
        payer: &AccountView,
        mint: &AccountView,
        owner: &Address,
        seeds: Option<&[CpiSigner]>,
    ) -> Result<Mut<'a, T>, Error> {
        let info: &'a mut AccountView = self.into();
        create_account_with_minimum_balance_signed(
            info,
            TokenAccount::LEN,
            &TOKEN_PROGRAM_ID,
            payer,
            rent,
            seeds.unwrap_or_default(),
        )?;

        InitializeAccount3 {
            account: info,
            mint,
            owner,
        }
        .invoke()?;

        Ok(Mut::from_raw_info(info))
    }

    fn create_associated_token_account(
        self,
        payer: &AccountView,
        mint: &AccountView,
        owner: &AccountView,
        system_program: &AccountView,
        token_program: &AccountView,
    ) -> Result<Mut<'a, T>, Error> {
        let info: &'a mut AccountView = self.into();
        Create {
            funding_account: payer,
            account: info,
            wallet: owner,
            mint,
            system_program,
            token_program,
        }
        .invoke()?;

        Ok(Mut::from_raw_info(info))
    }

    fn create_idempotent_associated_token_account(
        self,
        payer: &AccountView,
        mint: &AccountView,
        owner: &AccountView,
        system_program: &AccountView,
        token_program: &AccountView,
    ) -> Result<Mut<'a, T>, Error> {
        let info: &'a mut AccountView = self.into();
        CreateIdempotent {
            funding_account: payer,
            account: info,
            wallet: owner,
            mint,
            system_program,
            token_program,
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
