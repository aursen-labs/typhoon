#![no_std]

use quasar_lang::prelude::*;

declare_id!("Bench111111111111111111111111111111111111111");

#[program]
mod quasar_escrow {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn ping(_ctx: Ctx<Empty>) -> Result<(), ProgramError> {
        Ok(())
    }

    #[instruction(discriminator = 1)]
    pub fn log(_ctx: Ctx<Empty>) -> Result<(), ProgramError> {
        quasar_lang::prelude::log("Instruction: Log");
        Ok(())
    }

    #[instruction(discriminator = 2)]
    pub fn create_account(ctx: Ctx<CreateAccount>) -> Result<(), ProgramError> {
        ctx.accounts.handler()
    }

    #[instruction(discriminator = 3)]
    pub fn transfer(ctx: Ctx<Transfer>, amount: u64) -> Result<(), ProgramError> {
        ctx.accounts.handler(amount)
    }

    #[instruction(discriminator = 4)]
    pub fn unchecked_accounts(_ctx: Ctx<UncheckedAccounts>) -> Result<(), ProgramError> {
        Ok(())
    }

    #[instruction(discriminator = 5)]
    pub fn accounts(_ctx: Ctx<AccountsC>) -> Result<(), ProgramError> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Empty {}

#[derive(Accounts)]
pub struct CreateAccount {
    #[account(mut)]
    pub admin: Signer,
    #[account(
        init,
        payer = admin
    )]
    pub account: Account<Data>,
    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct Transfer {
    #[account(mut)]
    pub payer: Signer,
    #[account(mut)]
    pub account: SystemAccount,
    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct UncheckedAccounts {
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

#[derive(Accounts)]
pub struct AccountsC {
    pub account1: Account<Data>,
    /// CHECK: Benchmark intentionally reuses one initialized account for all slots.
    #[account(dup)]
    pub account2: Account<Data>,
    /// CHECK: Benchmark intentionally reuses one initialized account for all slots.
    #[account(dup)]
    pub account3: Account<Data>,
    /// CHECK: Benchmark intentionally reuses one initialized account for all slots.
    #[account(dup)]
    pub account4: Account<Data>,
    /// CHECK: Benchmark intentionally reuses one initialized account for all slots.
    #[account(dup)]
    pub account5: Account<Data>,
    /// CHECK: Benchmark intentionally reuses one initialized account for all slots.
    #[account(dup)]
    pub account6: Account<Data>,
    /// CHECK: Benchmark intentionally reuses one initialized account for all slots.
    #[account(dup)]
    pub account7: Account<Data>,
    /// CHECK: Benchmark intentionally reuses one initialized account for all slots.
    #[account(dup)]
    pub account8: Account<Data>,
    /// CHECK: Benchmark intentionally reuses one initialized account for all slots.
    #[account(dup)]
    pub account9: Account<Data>,
    /// CHECK: Benchmark intentionally reuses one initialized account for all slots.
    #[account(dup)]
    pub account10: Account<Data>,
}

impl CreateAccount {
    #[inline(always)]
    pub fn handler(&mut self) -> Result<(), ProgramError> {
        self.account.byte = 1;
        Ok(())
    }
}

impl Transfer {
    #[inline(always)]
    pub fn handler(&self, amount: u64) -> Result<(), ProgramError> {
        self.system_program
            .transfer(&self.payer, &self.account, amount)
            .invoke()
    }
}

#[account(discriminator = 1)]
pub struct Data {
    pub byte: u8,
}
