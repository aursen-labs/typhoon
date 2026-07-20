#![no_std]

use {
    bytemuck::{AnyBitPattern, NoUninit},
    typhoon::prelude::*,
};

nostd_panic_handler!();
no_allocator!();

program_id!("Bench111111111111111111111111111111111111111");

pub const ROUTER: EntryFn = basic_router! {
    0 => ping,
    1 => log,
    2 => create_account,
    3 => transfer,
    4 => unchecked_accounts,
    5 => accounts_c
};

entrypoint!(ROUTER);

fn ping() -> ProgramResult {
    Ok(())
}

fn log() -> ProgramResult {
    solana_program_log::log("Instruction: Log");
    Ok(())
}

fn create_account(mut ctx: CreateAccountContext) -> ProgramResult {
    ctx.account.mut_data()?.byte = 1;

    Ok(())
}

fn transfer(Arg(amount): Arg<[u8; 8]>, ctx: TransferContext) -> ProgramResult {
    ctx.admin
        .transfer(&ctx.account, u64::from_le_bytes(*amount))
}

fn unchecked_accounts(_ctx: UncheckedAccountsContext) -> ProgramResult {
    Ok(())
}

fn accounts_c(_ctx: AccountsContext) -> ProgramResult {
    Ok(())
}

#[context]
pub struct CreateAccountContext {
    pub admin: Mut<Signer>,
    #[constraint(
        init,
        payer = admin
    )]
    pub account: Mut<UncheckedSigner<Account<Data>>>,
    pub system_program: Program<System>,
}

#[context]
pub struct TransferContext {
    pub admin: Mut<Signer>,
    pub account: Mut<SystemAccount>,
    pub system_program: Program<System>,
}

#[context]
pub struct UncheckedAccountsContext {
    pub account1: UncheckedAccount,
    pub account2: UncheckedAccount,
    pub account3: UncheckedAccount,
    pub account4: UncheckedAccount,
    pub account5: UncheckedAccount,
    pub account6: UncheckedAccount,
    pub account7: UncheckedAccount,
    pub account8: UncheckedAccount,
    pub account9: UncheckedAccount,
    pub account10: UncheckedAccount,
}

#[context]
pub struct AccountsContext {
    pub account1: Account<Data>,
    pub account2: Account<Data>,
    pub account3: Account<Data>,
    pub account4: Account<Data>,
    pub account5: Account<Data>,
    pub account6: Account<Data>,
    pub account7: Account<Data>,
    pub account8: Account<Data>,
    pub account9: Account<Data>,
    pub account10: Account<Data>,
}

#[derive(NoUninit, AnyBitPattern, AccountState, Copy, Clone)]
#[repr(C)]
pub struct Data {
    pub byte: u8,
}
