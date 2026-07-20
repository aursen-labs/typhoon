use {
    escrow_interface::state::Escrow,
    typhoon::prelude::*,
    typhoon_token::{
        spl_instructions::{CloseAccount as SplCloseAccount, Transfer},
        AtaTokenProgram, Mint, SplCreateToken, TokenAccount, TokenProgram,
    },
};

#[context]
pub struct Take {
    pub taker: Mut<Signer>,
    pub maker: Mut<SystemAccount>,
    pub escrow: Mut<Account<Escrow>>,
    pub mint_a: Account<Mint>,
    pub mint_b: Account<Mint>,
    #[constraint(
        associated_token::mint = mint_a,
        associated_token::authority = escrow
    )]
    pub vault: Mut<Account<TokenAccount>>,
    #[constraint(
        init_if_needed
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker
    )]
    pub taker_ata_a: Mut<Account<TokenAccount>>,
    pub taker_ata_b: Mut<Account<TokenAccount>>,
    #[constraint(
        init_if_needed
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker
    )]
    pub maker_ata_b: Mut<Account<TokenAccount>>,
    pub ata_program: Program<AtaTokenProgram>,
    pub token_program: Program<TokenProgram>,
    pub system_program: Program<System>,
}

pub fn take(mut ctx: Take) -> ProgramResult {
    let escrow = ctx.escrow.data()?;
    let amount = { ctx.vault.data()?.amount() };
    let receive = { escrow.receive };

    let seed = escrow.seed.to_le_bytes();

    let bump = [escrow.bump];
    let seeds = seeds!(
        b"escrow",
        ctx.maker.address().as_ref(),
        seed.as_ref(),
        &bump
    );
    let signer = CpiSigner::from(&seeds);

    Transfer {
        from: ctx.vault.as_ref(),
        to: ctx.taker_ata_a.as_ref(),
        authority: ctx.escrow.as_ref(),
        amount,
    }
    .invoke_signed(&[signer.clone()])?;

    SplCloseAccount {
        account: ctx.vault.as_ref(),
        authority: ctx.escrow.as_ref(),
        destination: ctx.maker.as_ref(),
    }
    .invoke_signed(&[signer])?;

    drop(escrow);
    ctx.escrow.close(&ctx.maker)?;

    Transfer {
        from: ctx.taker_ata_b.as_ref(),
        to: ctx.maker_ata_b.as_ref(),
        authority: ctx.taker.as_ref(),
        amount: receive,
    }
    .invoke()?;

    Ok(())
}
