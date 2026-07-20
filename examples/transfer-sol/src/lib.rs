#![no_std]

use {
    bytemuck::{Pod, Zeroable},
    typhoon::prelude::*,
};

program_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

nostd_panic_handler!();
no_allocator!();
entrypoint!(ROUTER);

pub const ROUTER: EntryFn = basic_router! {
    0 => transfer_sol_with_cpi,
    1 => transfer_sol_with_program
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
#[repr(transparent)]
pub struct PodU64(pub [u8; 8]);

impl PodU64 {
    pub const fn from_primitive(n: u64) -> Self {
        Self(n.to_le_bytes())
    }
}
impl From<u64> for PodU64 {
    fn from(n: u64) -> Self {
        Self::from_primitive(n)
    }
}
impl From<PodU64> for u64 {
    fn from(pod: PodU64) -> Self {
        Self::from_le_bytes(pod.0)
    }
}

#[context]
pub struct Transfer {
    pub payer: Mut<Signer>,
    pub recipient: Mut<SystemAccount>,
}

#[context]
pub struct SystemProgram {
    pub system: Program<System>,
}

pub fn transfer_sol_with_cpi(
    Arg(amount): Arg<PodU64>,
    ctx: Transfer,
    _: SystemProgram,
) -> ProgramResult {
    ctx.payer.transfer(&ctx.recipient, (*amount).into())?;

    Ok(())
}

pub fn transfer_sol_with_program(Arg(amount): Arg<PodU64>, mut ctx: Transfer) -> ProgramResult {
    let amount = (*amount).into();
    ctx.payer.send(&mut ctx.recipient, amount)?;

    Ok(())
}
