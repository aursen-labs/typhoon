#![no_std]

mod error;

use {
    bytemuck::{AnyBitPattern, NoUninit},
    typhoon::prelude::*,
};

#[cfg(feature = "logging")]
pub type LogError = TestErrors;

program_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

nostd_panic_handler!();
no_allocator!();

#[context]
pub struct Init {
    pub payer: Mut<Signer>,
    #[constraint(
        init,
        payer = payer,
    )]
    pub counter: Mut<UncheckedSigner<Account<Counter>>>,
    pub system: Program<System>,
}

#[context]
pub struct CounterMut {
    pub counter: Mut<Account<Counter>>,
}

#[context]
pub struct Destination {
    pub destination: Mut<SystemAccount>,
}

#[context]
pub struct UnusedContext {
    pub random: SystemAccount,
}

// TODO add seeds and seeded

#[context]
#[args(value: u64)]
pub struct RandomContext {
    pub account: SystemAccount,
}

entrypoint!(ROUTER);

pub const ROUTER: EntryFn = basic_router! {
    0 => initialize,
    1 => increment,
    2 => close,
    3 => random_instruction,
};

pub fn initialize(_: Init) -> ProgramResult {
    Ok(())
}

pub fn increment(mut ctx: CounterMut) -> ProgramResult {
    ctx.counter.mut_data()?.count += 1;

    Ok(())
}

pub fn random_instruction(
    Arg(amount): Arg<AnotherStruct>,
    context: RandomContext,
) -> ProgramResult {
    Ok(())
}

pub fn close(
    CounterMut { mut counter }: CounterMut,
    Destination { mut destination }: Destination,
) -> ProgramResult {
    counter.close(&mut destination)?;

    Ok(())
}

#[derive(NoUninit, AnyBitPattern, AccountState, Copy, Clone)]
#[repr(C)]
pub struct Counter {
    pub count: u64,
    pub another: AnotherStruct,
}

#[derive(NoUninit, AnyBitPattern, Copy, Clone)]
#[repr(C)]
pub struct AnotherStruct {
    pub amount: u64,
}

pub struct ExcludedType {
    pub more_data: u32,
}
