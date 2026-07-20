#![no_std]

mod error;

use {
    crate::error::SeedsError,
    bytemuck::{AnyBitPattern, NoUninit},
    typhoon::prelude::*,
};

program_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

nostd_panic_handler!();
no_allocator!();
entrypoint!(ROUTER);

#[cfg(feature = "logging")]
pub type LogError = SeedsError;

pub const ROUTER: EntryFn = basic_router! {
    0 => initialize,
    1 => increment
};

fn pda_seeds<'a>() -> [&'a [u8]; 1] {
    [b"counter".as_ref()]
}

#[context]
pub struct Init {
    pub payer: Mut<Signer>,
    pub authority: Option<Signer>,
    #[constraint(
        init,
        payer = payer,
        space = Counter::SPACE,
        seeds = pda_seeds(),
        bump
    )]
    pub counter: Mut<Account<Counter>>,
    pub system: Program<System>,
}

#[context]
pub struct Increment {
    pub admin: Signer,
    #[constraint(
        has_one = admin @ SeedsError::InvalidOwner,
        seeds = [
            b"counter".as_ref(),
        ],
        bump = counter.data()?.bump,
    )]
    pub counter: Mut<Account<Counter>>,
}

pub fn initialize(mut ctx: Init) -> ProgramResult {
    assert!(ctx.authority.is_none());

    *ctx.counter.mut_data()? = Counter {
        bump: ctx.bumps.counter,
        admin: *ctx
            .authority
            .as_ref()
            .map(|a| a.address())
            .unwrap_or(ctx.payer.address()),
        count: 0,
        _padding: [0; 7],
    };

    Ok(())
}

pub fn increment(mut ctx: Increment) -> ProgramResult {
    ctx.counter.mut_data()?.count += 1;

    Ok(())
}

#[derive(AccountState, NoUninit, AnyBitPattern, Debug, Clone, Copy)]
#[no_space]
#[repr(C)]
pub struct Counter {
    pub count: u64,
    pub admin: Address,
    pub bump: u8,
    _padding: [u8; 7],
}

impl Counter {
    const SPACE: usize = 8 + core::mem::size_of::<Counter>();
}
