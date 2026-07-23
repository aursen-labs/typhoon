use {
    pinocchio::{
        address::{address_eq, declare_id, Address},
        error::ProgramError,
        hint, AccountView,
    },
    typhoon_accounts::*,
    typhoon_context::*,
    typhoon_context_macro::*,
    typhoon_errors::*,
    typhoon_program_id_macro::program_id,
    typhoon_token::Mint,
    typhoon_traits::*,
};

pub type ProgramResult<T = ()> = Result<T, Error>;

program_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// A non-`init` `Account<Mint>` with `mint::*` constraints must generate on-chain
// verification of the mint's authority / decimals / freeze authority.
#[context]
pub struct CheckMint {
    pub authority: Signer,
    #[constraint(
        mint::authority = authority,
        mint::decimals = 6,
        mint::freeze_authority = authority,
    )]
    pub mint: Account<Mint>,
}

pub fn main() {}
