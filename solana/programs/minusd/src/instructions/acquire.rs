use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintToChecked, Token, TokenAccount, TransferChecked};

use crate::errors::MinUsdError;
use crate::state::{Config, FrozenOwner};
use crate::{CONFIG_SEED, FROZEN_SEED, TOKEN_DECIMALS, VAULT_AUTHORITY_SEED, VAULT_SEED};

pub fn handler(ctx: Context<Acquire>, amount: u64) -> Result<()> {
    require!(amount > 0, MinUsdError::ZeroAmount);
    require!(!ctx.accounts.config.paused, MinUsdError::Paused);
    require!(
        ctx.accounts.recipient_minusd.owner != Pubkey::default(),
        MinUsdError::InvalidRecipient
    );
    require!(
        !FrozenOwner::is_frozen(&ctx.accounts.caller_freeze)?,
        MinUsdError::AccountIsFrozen
    );
    require!(
        !FrozenOwner::is_frozen(&ctx.accounts.recipient_freeze)?,
        MinUsdError::AccountIsFrozen
    );

    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.caller_usdc.to_account_info(),
                mint: ctx.accounts.mock_usdc_mint.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.caller.to_account_info(),
            },
        ),
        amount,
        TOKEN_DECIMALS,
    )?;

    let bump = [ctx.accounts.config.vault_authority_bump];
    let signer_seeds: &[&[u8]] = &[VAULT_AUTHORITY_SEED, &bump];

    token::mint_to_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintToChecked {
                mint: ctx.accounts.minusd_mint.to_account_info(),
                to: ctx.accounts.recipient_minusd.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &[signer_seeds],
        ),
        amount,
        TOKEN_DECIMALS,
    )?;

    emit!(Acquired {
        caller: ctx.accounts.caller.key(),
        recipient: ctx.accounts.recipient_minusd.owner,
        amount,
    });
    msg!(
        "acquire caller={} recipient={} amount={}",
        ctx.accounts.caller.key(),
        ctx.accounts.recipient_minusd.owner,
        amount
    );
    Ok(())
}

#[event]
pub struct Acquired {
    pub caller: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
}

#[derive(Accounts)]
pub struct Acquire<'info> {
    pub caller: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = mock_usdc_mint,
        has_one = minusd_mint,
        has_one = vault
    )]
    pub config: Account<'info, Config>,

    pub mock_usdc_mint: Account<'info, Mint>,

    #[account(mut)]
    pub minusd_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = mock_usdc_mint,
        token::authority = caller
    )]
    pub caller_usdc: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump,
        token::mint = mock_usdc_mint,
        token::authority = vault_authority
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut, token::mint = minusd_mint)]
    pub recipient_minusd: Account<'info, TokenAccount>,

    /// CHECK: PDA mint authority.
    #[account(seeds = [VAULT_AUTHORITY_SEED], bump = config.vault_authority_bump)]
    pub vault_authority: UncheckedAccount<'info>,

    /// CHECK: empty = not frozen; seeds bind this to the caller.
    #[account(seeds = [FROZEN_SEED, caller.key().as_ref()], bump)]
    pub caller_freeze: UncheckedAccount<'info>,

    /// CHECK: empty = not frozen; seeds bind this to the recipient owner.
    #[account(seeds = [FROZEN_SEED, recipient_minusd.owner.as_ref()], bump)]
    pub recipient_freeze: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}
