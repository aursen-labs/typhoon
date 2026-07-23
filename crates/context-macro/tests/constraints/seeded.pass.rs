use {
    bytemuck::{AnyBitPattern, NoUninit},
    pinocchio::{
        address::Address,
        cpi::Seed,
        error::ProgramError,
        instruction::seeds,
        AccountView,
    },
    typhoon_account_macro::*,
    typhoon_accounts::*,
    typhoon_context::*,
    typhoon_context_macro::*,
    typhoon_errors::*,
    typhoon_program_id_macro::program_id,
    typhoon_traits::*,
};

pub type ProgramResult<T = ()> = Result<T, Error>;

program_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[derive(NoUninit, AnyBitPattern, AccountState, Copy, Clone)]
#[repr(C)]
pub struct Counter {
    #[key]
    pub admin: Address,
    #[key]
    pub counter_id: u64,
    pub count: u64,
    pub bump: u8,
    pub _padding: [u8; 15],
}

#[context]
#[args(admin: Address, counter_id: u64)]
pub struct InitContext {
    pub payer: Mut<Signer>,
    #[constraint(
        seeded = [&args.admin, &args.counter_id],
        bump
    )]
    pub counter: Mut<Account<Counter>>,
    pub system: Program<System>,
}

#[context]
pub struct IncrementContext {
    pub payer: Mut<Signer>,
    #[constraint(seeded, bump = counter.data()?.bump)]
    pub counter: Mut<Account<Counter>>,
}

pub fn main() {}
