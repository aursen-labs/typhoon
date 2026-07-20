use {
    escrow_interface::{state::Escrow, MakeArgs},
    typhoon::prelude::*,
    typhoon_token::{spl_instructions::Transfer, *},
};

#[context]
#[args(MakeArgs)]
pub struct Make {
    pub maker: Mut<Signer>,
    #[constraint(
        init,
        payer = maker,
        seeds = [b"escrow", maker.address().as_ref(), &args.seed.to_le_bytes()],
        bump
    )]
    pub escrow: Mut<Account<Escrow>>,
    pub mint_a: Account<Mint>,
    pub mint_b: Account<Mint>,
    pub maker_ata_a: Mut<Account<TokenAccount>>,
    #[constraint(
        init_if_needed,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow
    )]
    pub vault: Mut<Account<TokenAccount>>,
    pub ata_program: Program<AtaTokenProgram>,
    pub token_program: Program<TokenProgram>,
    pub system_program: Program<System>,
}

pub fn make(mut ctx: Make) -> ProgramResult {
    let mut escrow_state = ctx.escrow.mut_data()?;

    *escrow_state = Escrow {
        maker: *ctx.maker.address(),
        mint_a: *ctx.mint_a.address(),
        mint_b: *ctx.mint_b.address(),
        seed: ctx.args.seed,
        receive: ctx.args.receive,
        bump: ctx.bumps.escrow,
    };

    Transfer::new(
        ctx.maker_ata_a.as_ref(),
        ctx.vault.as_ref(),
        ctx.maker.as_ref(),
        ctx.args.amount,
    )
    .invoke()?;

    Ok(())
}
