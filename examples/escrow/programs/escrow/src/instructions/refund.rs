use {
    escrow_interface::state::Escrow,
    typhoon::prelude::*,
    typhoon_token::{
        spl_instructions::{CloseAccount as SplCloseAccount, Transfer},
        TokenAccount, TokenProgram,
    },
};

#[context]
pub struct Refund {
    pub maker: Mut<Signer>,
    pub escrow: Mut<Account<Escrow>>,
    pub mint_a: UncheckedAccount,
    #[constraint(
        associated_token::mint = mint_a,
        associated_token::authority = escrow
    )]
    pub vault: Mut<Account<TokenAccount>>,
    pub maker_ata_a: Mut<Account<TokenAccount>>,
    pub token_program: Program<TokenProgram>,
    pub system_program: Program<System>,
}

pub fn refund(mut ctx: Refund) -> ProgramResult {
    let escrow = ctx.escrow.data()?;
    let seed = escrow.seed.to_le_bytes();
    let seeds = seeds!(b"escrow", ctx.maker.address().as_ref(), seed.as_ref());
    let signer = CpiSigner::from(&seeds);

    let amount = { ctx.vault.data()?.amount() };

    Transfer::new(
        ctx.vault.as_ref(),
        ctx.maker_ata_a.as_ref(),
        ctx.escrow.as_ref(),
        amount,
    )
    .invoke_signed(&[signer.clone()])?;

    SplCloseAccount::new(
        ctx.vault.as_ref(),
        ctx.maker.as_ref(),
        ctx.escrow.as_ref(),
    )
    .invoke_signed(&[signer])?;

    drop(escrow);
    ctx.escrow.close(&mut ctx.maker)?;

    Ok(())
}
