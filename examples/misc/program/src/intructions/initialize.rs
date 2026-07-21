use {
    misc_interface::{pda::RANDOM_PDA, state::RandomData},
    typhoon::prelude::*,
};

#[context]
pub struct Initialize {
    pub payer: Mut<Signer>,
    #[constraint(
        init,
        payer = payer,
        seeds = [b"random"],
        bump = RANDOM_PDA.1
    )]
    pub account: Mut<Account<RandomData>>,
    pub system_program: Program<System>,
}

pub fn initialize(mut context: Initialize) -> ProgramResult {
    context.account.mut_data()?.counter = 1;
    Ok(())
}
