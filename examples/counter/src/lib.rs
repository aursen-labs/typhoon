#![no_std]

use {
    bytemuck::{AnyBitPattern, NoUninit},
    typhoon::prelude::*,
};

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

entrypoint!(ROUTER);

pub const ROUTER: EntryFn = basic_router! {
    0 => initialize,
    1 => increment,
    2 => close,
};

pub fn initialize(_: Init) -> ProgramResult {
    Ok(())
}

pub fn increment(mut ctx: CounterMut) -> ProgramResult {
    ctx.counter.mut_data()?.count += 1;

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
}
