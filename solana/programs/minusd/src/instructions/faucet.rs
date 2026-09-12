use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintToChecked, Token, TokenAccount};

use crate::errors::MinUsdError;
use crate::state::Config;
use crate::{CONFIG_SEED, TOKEN_DECIMALS, VAULT_AUTHORITY_SEED};

pub fn handler(ctx: Context<MintMockUsdc>, amount: u64) -> Result<()> {
    require!(amount > 0, MinUsdError::ZeroAmount);
    require!(
        ctx.accounts.recipient_usdc.owner != Pubkey::default(),
        MinUsdError::InvalidRecipient
    );

    let bump = [ctx.accounts.config.vault_authority_bump];
    let signer_seeds: &[&[u8]] = &[VAULT_AUTHORITY_SEED, &bump];

    token::mint_to_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintToChecked {
                mint: ctx.accounts.mock_usdc_mint.to_account_info(),
                to: ctx.accounts.recipient_usdc.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &[signer_seeds],
        ),
        amount,
        TOKEN_DECIMALS,
    )?;

    msg!(
        "faucet recipient={} amount={}",
        ctx.accounts.recipient_usdc.owner,
        amount
    );
    Ok(())
}

#[derive(Accounts)]
pub struct MintMockUsdc<'info> {
    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = mock_usdc_mint
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub mock_usdc_mint: Account<'info, Mint>,

    #[account(mut, token::mint = mock_usdc_mint)]
    pub recipient_usdc: Account<'info, TokenAccount>,

    /// CHECK: PDA mint authority.
    #[account(seeds = [VAULT_AUTHORITY_SEED], bump = config.vault_authority_bump)]
    pub vault_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}
