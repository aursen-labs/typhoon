use {
    crate::Mint,
    pinocchio::{cpi::Signer as CpiSigner, sysvars::rent::Rent, AccountView, Address},
    pinocchio_token::{instructions::InitializeMint2, ID as TOKEN_PROGRAM_ID},
    typhoon_accounts::{Account, Mut, Signer, SignerCheck, ValidateView, WritableAccount},
    typhoon_errors::Error,
    typhoon_utility::create_account_with_minimum_balance_signed,
};

pub trait SplCreateMint<'a, T>
where
    Self: Sized + Into<&'a mut AccountView>,
    T: ValidateView,
{
    #[inline]
    fn create_mint(
        self,
        rent: &Rent,
        payer: &impl WritableAccount,
        mint_authority: &Address,
        decimals: u8,
        freeze_authority: Option<&Address>,
        seeds: Option<&[CpiSigner]>,
    ) -> Result<Mut<'a, T>, Error> {
        let info: &'a mut AccountView = self.into();
        create_account_with_minimum_balance_signed(
            info,
            Mint::LEN,
            &TOKEN_PROGRAM_ID,
            payer.as_ref(),
            rent,
            seeds.unwrap_or_default(),
        )?;

        InitializeMint2 {
            mint: info,
            mint_authority,
            decimals,
            freeze_authority,
        }
        .invoke()?;

        Mut::try_from(info)
    }
}

impl<'a> SplCreateMint<'a, Account<'a, Mint>> for &'a mut AccountView {}

impl<'a, C> SplCreateMint<'a, Signer<'a, Account<'a, Mint>, C>> for &'a mut AccountView where
    C: SignerCheck
{
}
