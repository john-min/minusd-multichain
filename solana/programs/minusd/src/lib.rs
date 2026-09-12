//! MinUSD Solana lifecycle.
//!
//! Educational, testnet-only prototype. It is unaudited, has no monetary value,
//! is not backed by real reserves, and is not redeemable for fiat.

use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

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
        instructions::initialize::handler(ctx, admin, pauser, compliance)
    }

    pub fn mint_mock_usdc(ctx: Context<MintMockUsdc>, amount: u64) -> Result<()> {
        instructions::faucet::handler(ctx, amount)
    }

    pub fn acquire(ctx: Context<Acquire>, amount: u64) -> Result<()> {
        instructions::acquire::handler(ctx, amount)
    }

    pub fn redeem(ctx: Context<Redeem>, amount: u64) -> Result<()> {
        instructions::redeem::handler(ctx, amount)
    }

    pub fn pause(ctx: Context<SetPaused>) -> Result<()> {
        instructions::pause::pause(ctx)
    }

    pub fn unpause(ctx: Context<SetPaused>) -> Result<()> {
        instructions::pause::unpause(ctx)
    }

    pub fn freeze(ctx: Context<SetFrozen>) -> Result<()> {
        instructions::freeze::freeze(ctx)
    }

    pub fn unfreeze(ctx: Context<SetFrozen>) -> Result<()> {
        instructions::freeze::unfreeze(ctx)
    }

    pub fn set_pauser(ctx: Context<AdminSet>, new_pauser: Pubkey) -> Result<()> {
        instructions::admin::set_pauser(ctx, new_pauser)
    }

    pub fn set_compliance(ctx: Context<AdminSet>, new_compliance: Pubkey) -> Result<()> {
        instructions::admin::set_compliance(ctx, new_compliance)
    }
}
