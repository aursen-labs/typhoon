#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use typhoon::prelude::*;

program_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// nostd_panic_handler!();
// no_allocator!();
entrypoint!();

pub const ROUTER: EntryFn = basic_router! {
    0 => pull_lever
};

pub fn pull_lever(ctx: PullLever, Arg(name): Arg<[u8; 16]>) -> ProgramResult {
    let last_char = name.iter().position(|&x| x == 0).unwrap_or(name.len());
    crate::lever_cpi::SwitchPower {
        power: ctx.power.as_ref(),
        name: core::str::from_utf8(&name[..last_char])
            .map_err(|_| ProgramError::InvalidInstructionData)?,
        details: &crate::lever_cpi::Details { info: "123".into() },
    }
    .invoke()
}

#[context]
pub struct PullLever {
    pub power: Mut<Account<crate::lever_cpi::PowerStatus>>,
    pub lever_program: Program<crate::lever_cpi::LeverProgram>,
}

anchor_cpi!("../../idls/lever.json");
