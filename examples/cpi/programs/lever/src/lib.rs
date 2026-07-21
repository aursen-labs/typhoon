#![no_std]

use {lever_interface::PowerStatus, solana_program_log::log, typhoon::prelude::*};

nostd_panic_handler!();
no_allocator!();
entrypoint!(ROUTER);

pub const ROUTER: EntryFn = basic_router! {
    0 => initialize,
    1 => switch_power,
    2 => check_power
};

pub fn initialize(_ctx: InitializeLever) -> ProgramResult {
    Ok(())
}

pub fn switch_power(mut ctx: SetPowerStatus) -> ProgramResult {
    let mut power = ctx.power.mut_data()?;
    power.change_status();

    match power.is_on() {
        true => log("The power is now on."),
        false => log("The power is now off!"),
    };
    Ok(())
}

pub fn check_power(ctx: CheckStatus) -> ProgramResult<u8> {
    let power = ctx.power.as_ref().unwrap().data()?;

    match power.is_on() {
        true => log("The power is now on."),
        false => log("The power is now off!"),
    };

    Ok(1)
}

#[context]
pub struct InitializeLever {
    #[constraint(
        init,
        payer = user
    )]
    pub power: Mut<UncheckedSigner<Account<PowerStatus>>>,
    pub user: Mut<Signer>,
    pub system_program: Program<System>,
}

#[context]
pub struct SetPowerStatus {
    pub power: Mut<Account<PowerStatus>>,
}

#[context]
#[args(random: u64)]
pub struct CheckStatus {
    pub power: Option<Account<PowerStatus>>,
}
