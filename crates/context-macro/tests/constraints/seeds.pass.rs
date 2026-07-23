use pinocchio::{
    address::Address,
    cpi::Signer as CpiSigner,
    error::ProgramError,
    instruction::seeds,
    sysvars::{rent::Rent, Sysvar},
    AccountView,
};
use {
    bytemuck::{AnyBitPattern, NoUninit},
    typhoon_account_macro::*,
    typhoon_accounts::*,
    typhoon_context::*,
    typhoon_context_macro::*,
    typhoon_errors::*,
    typhoon_program_id_macro::program_id,
    typhoon_traits::*,
    typhoon_utility_traits::CreateAccountCpi,
};

pub type ProgramResult<T = ()> = Result<T, Error>;

program_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[derive(NoUninit, AnyBitPattern, AccountState, Copy, Clone)]
#[repr(C)]
pub struct Counter {
    pub count: u64,
    pub counter_id: u64,
}

#[context]
#[args(admin: Address, counter_id: u64)]
pub struct InitContext {
    pub payer: Mut<Signer>,
    #[constraint(
        init_if_needed,
        payer = payer,
        seeds = [args.admin.as_ref(), &args.counter_id.to_le_bytes()],
        bump
    )]
    pub counter: Mut<Account<Counter>>,
    pub system: Program<System>,
}

#[context]
#[args(admin: Address, counter_id: u64)]
pub struct OtherContext {
    pub payer: Mut<Signer>,
    #[constraint(
        seeds = [args.admin.as_ref(), &args.counter_id.to_le_bytes()],
        bump
    )]
    pub counter: Mut<Account<Counter>>,
    pub system: Program<System>,
}

#[context]
#[args(admin: Address, counter_id: u64)]
pub struct InitSequentialCounterContext {
    pub payer: Mut<Signer>,
    pub previous_counter: Mut<Account<Counter>>,
    #[constraint(
        init,
        payer = payer,
        seeds = [args.admin.as_ref(), &previous_counter.data()?.counter_id.to_le_bytes()],
        bump
    )]
    pub counter: Mut<Account<Counter>>,
    pub system: Program<System>,
}

pub fn main() {}
