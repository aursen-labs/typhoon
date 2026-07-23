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

entrypoint!(ROUTER);

pub const ROUTER: EntryFn = basic_router! {
    0 => initialize,
    1 => increment,
};

pub fn initialize(_: Init) -> ProgramResult {
    Ok(())
}

pub fn increment(mut ctx: CounterMut) -> ProgramResult {
    let count = {
        let mut data = ctx.counter.mut_data()?;
        data.count += 1;
        data.count
    };

    // Emit an event to the transaction logs. The payload is the variant's
    // discriminator (its index, as a `u8`) followed by its packed fields, and
    // shows up as a `Program data:` log line that clients can decode against
    // the IDL.
    CounterEvent::Incremented { count }.emit();

    Ok(())
}

#[derive(NoUninit, AnyBitPattern, AccountState, Copy, Clone)]
#[repr(C)]
pub struct Counter {
    pub count: u64,
}

#[event]
pub enum CounterEvent {
    Incremented { count: u64 },
}
