use {
    bytemuck::{AnyBitPattern, NoUninit},
    pinocchio::{
        address::{self, address_eq, declare_id, Address},
        cpi::{Seed, Signer as CpiSigner},
        error::ProgramError,
        hint,
        instruction::seeds,
        sysvars::{rent::Rent, Sysvar},
        AccountView,
    },
    typhoon_account_macro::*,
    typhoon_accounts::*,
    typhoon_context::*,
    typhoon_context_macro::*,
    typhoon_errors::*,
    typhoon_program_id_macro::program_id,
    typhoon_token::Mint,
    typhoon_traits::*,
    typhoon_utility_traits::CreateAccountCpi,
};

pub type ProgramResult<T = ()> = Result<T, Error>;

program_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// A non-`init` `Account<Mint>` with `mint::*` constraints must generate on-chain
// verification of the mint's authority / decimals / freeze authority.
#[context]
pub struct CheckMint {
    pub authority: Signer,
    #[constraint(
        mint::authority = authority.address(),
        mint::decimals = 6,
        mint::freeze_authority = authority.address(),
    )]
    pub mint: Account<Mint>,
}

pub fn main() {}
