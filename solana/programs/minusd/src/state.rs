use anchor_lang::prelude::*;

/// Program config. One PDA; roles and mint/vault identities live here.
#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub pauser: Pubkey,
    pub compliance: Pubkey,
    pub mock_usdc_mint: Pubkey,
    pub minusd_mint: Pubkey,
    pub vault: Pubkey,
    pub paused: bool,
    pub bump: u8,
    pub vault_authority_bump: u8,
}

/// Owner-level freeze flag. Complements SPL token-account freeze.
#[account]
#[derive(InitSpace)]
pub struct FrozenOwner {
    pub owner: Pubkey,
    pub frozen: bool,
    pub bump: u8,
}

impl FrozenOwner {
    /// Uninitialized PDA (empty data) means the owner is not frozen.
    pub fn is_frozen(account: &UncheckedAccount) -> Result<bool> {
        if account.data_is_empty() {
            return Ok(false);
        }
        let mut data: &[u8] = &account.try_borrow_data()?;
        let rec = FrozenOwner::try_deserialize(&mut data)?;
        Ok(rec.frozen)
    }
}
