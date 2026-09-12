use anchor_lang::prelude::*;
use anchor_spl::token::{self, FreezeAccount, Mint, ThawAccount, Token, TokenAccount};

use crate::errors::MinUsdError;
use crate::state::{Config, FrozenOwner};
use crate::{CONFIG_SEED, FROZEN_SEED, VAULT_AUTHORITY_SEED};

pub fn freeze(ctx: Context<SetFrozen>) -> Result<()> {
    require!(
        ctx.accounts.owner.key() != Pubkey::default(),
        MinUsdError::InvalidRecipient
    );
    let rec = &mut ctx.accounts.frozen_owner;
    require!(!rec.frozen, MinUsdError::AlreadyFrozen);

    rec.owner = ctx.accounts.owner.key();
    rec.frozen = true;
    rec.bump = ctx.bumps.frozen_owner;

    let bump = [ctx.accounts.config.vault_authority_bump];
    let signer_seeds: &[&[u8]] = &[VAULT_AUTHORITY_SEED, &bump];

    token::freeze_account(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            FreezeAccount {
                account: ctx.accounts.minusd_account.to_account_info(),
                mint: ctx.accounts.minusd_mint.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &[signer_seeds],
        ),
    )?;

    emit!(AccountFrozen {
        account: ctx.accounts.owner.key(),
        operator: ctx.accounts.compliance.key(),
    });
    msg!(
        "freeze owner={} ata={}",
        ctx.accounts.owner.key(),
        ctx.accounts.minusd_account.key()
    );
    Ok(())
}

pub fn unfreeze(ctx: Context<SetFrozen>) -> Result<()> {
    require!(
        ctx.accounts.owner.key() != Pubkey::default(),
        MinUsdError::InvalidRecipient
    );
    let rec = &mut ctx.accounts.frozen_owner;
    require!(rec.frozen, MinUsdError::NotFrozen);

    rec.frozen = false;

    let bump = [ctx.accounts.config.vault_authority_bump];
    let signer_seeds: &[&[u8]] = &[VAULT_AUTHORITY_SEED, &bump];

    token::thaw_account(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            ThawAccount {
                account: ctx.accounts.minusd_account.to_account_info(),
                mint: ctx.accounts.minusd_mint.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &[signer_seeds],
        ),
    )?;

    emit!(AccountUnfrozen {
        account: ctx.accounts.owner.key(),
        operator: ctx.accounts.compliance.key(),
    });
    msg!(
        "unfreeze owner={} ata={}",
        ctx.accounts.owner.key(),
        ctx.accounts.minusd_account.key()
    );
    Ok(())
}

#[event]
pub struct AccountFrozen {
    pub account: Pubkey,
    pub operator: Pubkey,
}

#[event]
pub struct AccountUnfrozen {
    pub account: Pubkey,
    pub operator: Pubkey,
}

#[derive(Accounts)]
pub struct SetFrozen<'info> {
    #[account(mut)]
    pub compliance: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = minusd_mint,
        has_one = compliance
    )]
    pub config: Account<'info, Config>,

    /// CHECK: wallet whose MINUSD ATA is being frozen/thawed.
    pub owner: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = compliance,
        space = 8 + FrozenOwner::INIT_SPACE,
        seeds = [FROZEN_SEED, owner.key().as_ref()],
        bump
    )]
    pub frozen_owner: Account<'info, FrozenOwner>,

    #[account(
        mut,
        token::mint = minusd_mint,
        token::authority = owner
    )]
    pub minusd_account: Account<'info, TokenAccount>,

    pub minusd_mint: Account<'info, Mint>,

    /// CHECK: PDA freeze authority.
    #[account(seeds = [VAULT_AUTHORITY_SEED], bump = config.vault_authority_bump)]
    pub vault_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
