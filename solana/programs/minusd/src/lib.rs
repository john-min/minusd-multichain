//! MinUSD Solana lifecycle.
//!
//! Educational, testnet-only prototype. It is unaudited, has no monetary value,
//! is not backed by real reserves, and is not redeemable for fiat.
//!
//! Account structs live in this file so Anchor's `#[program]` client-account
//! imports resolve. Helpers stay in `state` / `errors`.

use anchor_lang::prelude::*;
use anchor_spl::token::{
    self, Burn, FreezeAccount, Mint, MintTo, ThawAccount, Token, TokenAccount, TransferChecked,
};

pub mod errors;
pub mod state;

use errors::MinUsdError;
use state::{Config, FrozenOwner};

declare_id!("fLsZq7gE9KmSGPm5CR78FcfGtfvFGXR2kYPtsrbxNRw");

pub const TOKEN_DECIMALS: u8 = 6;
pub const CONFIG_SEED: &[u8] = b"config";
pub const VAULT_AUTHORITY_SEED: &[u8] = b"vault_authority";
pub const VAULT_SEED: &[u8] = b"vault";
pub const MOCK_USDC_MINT_SEED: &[u8] = b"mock_usdc_mint";
pub const MINUSD_MINT_SEED: &[u8] = b"minusd_mint";
pub const FROZEN_SEED: &[u8] = b"frozen";

#[program]
pub mod minusd {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        admin: Pubkey,
        pauser: Pubkey,
        compliance: Pubkey,
    ) -> Result<()> {
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

    pub fn mint_mock_usdc(ctx: Context<MintMockUsdc>, amount: u64) -> Result<()> {
        require!(amount > 0, MinUsdError::ZeroAmount);
        require!(
            ctx.accounts.recipient_usdc.owner != Pubkey::default(),
            MinUsdError::InvalidRecipient
        );

        let bump = [ctx.accounts.config.vault_authority_bump];
        let signer_seeds: &[&[u8]] = &[VAULT_AUTHORITY_SEED, &bump];

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mock_usdc_mint.to_account_info(),
                    to: ctx.accounts.recipient_usdc.to_account_info(),
                    authority: ctx.accounts.vault_authority.to_account_info(),
                },
                &[signer_seeds],
            ),
            amount,
        )?;

        msg!(
            "faucet recipient={} amount={}",
            ctx.accounts.recipient_usdc.owner,
            amount
        );
        Ok(())
    }

    pub fn acquire(ctx: Context<Acquire>, amount: u64) -> Result<()> {
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

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.minusd_mint.to_account_info(),
                    to: ctx.accounts.recipient_minusd.to_account_info(),
                    authority: ctx.accounts.vault_authority.to_account_info(),
                },
                &[signer_seeds],
            ),
            amount,
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

    pub fn redeem(ctx: Context<Redeem>, amount: u64) -> Result<()> {
        require!(amount > 0, MinUsdError::ZeroAmount);
        require!(!ctx.accounts.config.paused, MinUsdError::Paused);
        require!(
            ctx.accounts.recipient_usdc.owner != Pubkey::default(),
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

        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.minusd_mint.to_account_info(),
                    from: ctx.accounts.caller_minusd.to_account_info(),
                    authority: ctx.accounts.caller.to_account_info(),
                },
            ),
            amount,
        )?;

        let bump = [ctx.accounts.config.vault_authority_bump];
        let signer_seeds: &[&[u8]] = &[VAULT_AUTHORITY_SEED, &bump];

        token::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.vault.to_account_info(),
                    mint: ctx.accounts.mock_usdc_mint.to_account_info(),
                    to: ctx.accounts.recipient_usdc.to_account_info(),
                    authority: ctx.accounts.vault_authority.to_account_info(),
                },
                &[signer_seeds],
            ),
            amount,
            TOKEN_DECIMALS,
        )?;

        emit!(Redeemed {
            caller: ctx.accounts.caller.key(),
            recipient: ctx.accounts.recipient_usdc.owner,
            amount,
        });
        msg!(
            "redeem caller={} recipient={} amount={}",
            ctx.accounts.caller.key(),
            ctx.accounts.recipient_usdc.owner,
            amount
        );
        Ok(())
    }

    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        ctx.accounts.config.paused = true;
        emit!(PausedEvent {
            pauser: ctx.accounts.pauser.key(),
        });
        msg!("paused by {}", ctx.accounts.pauser.key());
        Ok(())
    }

    pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
        ctx.accounts.config.paused = false;
        emit!(UnpausedEvent {
            pauser: ctx.accounts.pauser.key(),
        });
        msg!("unpaused by {}", ctx.accounts.pauser.key());
        Ok(())
    }

    pub fn freeze(ctx: Context<Freeze>) -> Result<()> {
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

    pub fn unfreeze(ctx: Context<Unfreeze>) -> Result<()> {
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

    pub fn set_pauser(ctx: Context<SetPauser>, new_pauser: Pubkey) -> Result<()> {
        require!(new_pauser != Pubkey::default(), MinUsdError::InvalidRecipient);
        ctx.accounts.config.pauser = new_pauser;
        msg!("set_pauser {}", new_pauser);
        Ok(())
    }

    pub fn set_compliance(ctx: Context<SetCompliance>, new_compliance: Pubkey) -> Result<()> {
        require!(new_compliance != Pubkey::default(), MinUsdError::InvalidRecipient);
        ctx.accounts.config.compliance = new_compliance;
        msg!("set_compliance {}", new_compliance);
        Ok(())
    }
}

#[event]
pub struct Acquired {
    pub caller: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
}

#[event]
pub struct Redeemed {
    pub caller: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
}

#[event]
pub struct PausedEvent {
    pub pauser: Pubkey,
}

#[event]
pub struct UnpausedEvent {
    pub pauser: Pubkey,
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

#[derive(Accounts)]
pub struct Redeem<'info> {
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
        token::mint = minusd_mint,
        token::authority = caller
    )]
    pub caller_minusd: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump,
        token::mint = mock_usdc_mint,
        token::authority = vault_authority
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut, token::mint = mock_usdc_mint)]
    pub recipient_usdc: Account<'info, TokenAccount>,

    /// CHECK: PDA vault authority.
    #[account(seeds = [VAULT_AUTHORITY_SEED], bump = config.vault_authority_bump)]
    pub vault_authority: UncheckedAccount<'info>,

    /// CHECK: empty = not frozen; seeds bind this to the caller.
    #[account(seeds = [FROZEN_SEED, caller.key().as_ref()], bump)]
    pub caller_freeze: UncheckedAccount<'info>,

    /// CHECK: empty = not frozen; seeds bind this to the recipient owner.
    #[account(seeds = [FROZEN_SEED, recipient_usdc.owner.as_ref()], bump)]
    pub recipient_freeze: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Pause<'info> {
    pub pauser: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = pauser
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct Unpause<'info> {
    pub pauser: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = pauser
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct Freeze<'info> {
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

#[derive(Accounts)]
pub struct Unfreeze<'info> {
    #[account(mut)]
    pub compliance: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = minusd_mint,
        has_one = compliance
    )]
    pub config: Account<'info, Config>,

    /// CHECK: wallet whose MINUSD ATA is being thawed.
    pub owner: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [FROZEN_SEED, owner.key().as_ref()],
        bump = frozen_owner.bump
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
}

#[derive(Accounts)]
pub struct SetPauser<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = admin
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct SetCompliance<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = admin
    )]
    pub config: Account<'info, Config>,
}
