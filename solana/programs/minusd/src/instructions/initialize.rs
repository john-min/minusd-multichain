use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::errors::MinUsdError;
use crate::state::Config;
use crate::{
    CONFIG_SEED, MINUSD_MINT_SEED, MOCK_USDC_MINT_SEED, TOKEN_DECIMALS, VAULT_AUTHORITY_SEED,
    VAULT_SEED,
};

pub fn handler(ctx: Context<Initialize>, admin: Pubkey, pauser: Pubkey, compliance: Pubkey) -> Result<()> {
    require!(admin != Pubkey::default(), MinUsdError::InvalidRecipient);
    require!(pauser != Pubkey::default(), MinUsdError::InvalidRecipient);
    require!(compliance != Pubkey::default(), MinUsdError::InvalidRecipient);
    require!(
        ctx.accounts.mock_usdc_mint.decimals == TOKEN_DECIMALS,
        MinUsdError::DecimalMismatch
    );
    require!(
        ctx.accounts.minusd_mint.decimals == TOKEN_DECIMALS,
        MinUsdError::DecimalMismatch
    );

    let config = &mut ctx.accounts.config;
    config.admin = admin;
    config.pauser = pauser;
    config.compliance = compliance;
    config.mock_usdc_mint = ctx.accounts.mock_usdc_mint.key();
    config.minusd_mint = ctx.accounts.minusd_mint.key();
    config.vault = ctx.accounts.vault.key();
    config.paused = false;
    config.bump = ctx.bumps.config;
    config.vault_authority_bump = ctx.bumps.vault_authority;

    msg!(
        "initialized admin={} pauser={} compliance={} mock_usdc={} minusd={} vault={}",
        admin,
        pauser,
        compliance,
        config.mock_usdc_mint,
        config.minusd_mint,
        config.vault
    );
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + Config::INIT_SPACE,
        seeds = [CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, Config>,

    /// CHECK: PDA used as mint, freeze, and vault authority. No data.
    #[account(seeds = [VAULT_AUTHORITY_SEED], bump)]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = vault_authority,
        seeds = [MOCK_USDC_MINT_SEED],
        bump
    )]
    pub mock_usdc_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = vault_authority,
        mint::freeze_authority = vault_authority,
        seeds = [MINUSD_MINT_SEED],
        bump
    )]
    pub minusd_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        token::mint = mock_usdc_mint,
        token::authority = vault_authority,
        seeds = [VAULT_SEED],
        bump
    )]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
