use {
    crate::Mint,
    pinocchio::{cpi::Signer as CpiSigner, sysvars::rent::Rent, AccountView, Address},
    pinocchio_token::{instructions::InitializeMint2, ID as TOKEN_PROGRAM_ID},
    typhoon_accounts::{
        Account, Mut, ReadableAccount, Signer, SignerCheck, SystemAccount, UncheckedAccount,
        WritableAccount,
    },
    typhoon_errors::Error,
    typhoon_utility::create_account_with_minimum_balance_signed,
};

pub trait SplCreateMint<'a, T: ReadableAccount>
where
    Self: Sized + Into<&'a AccountView>,
    T: ReadableAccount + TryFrom<&'a AccountView, Error = Error>,
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
    ) -> Result<Mut<T>, Error> {
        let info = self.into();
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

macro_rules! impl_trait {
    ($origin: ty) => {
        impl<'a> SplCreateMint<'a, Account<'a, Mint>> for $origin {}
        impl<'a, C> SplCreateMint<'a, Signer<'a, Account<'a, Mint>, C>> for $origin where
            C: SignerCheck
        {
        }
    };
}

impl_trait!(&'a AccountView);
impl_trait!(SystemAccount<'a>);
impl_trait!(UncheckedAccount<'a>);
