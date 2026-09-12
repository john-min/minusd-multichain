use anchor_lang::prelude::*;

use crate::errors::MinUsdError;
use crate::state::Config;
use crate::CONFIG_SEED;

pub fn set_pauser(ctx: Context<AdminSet>, new_pauser: Pubkey) -> Result<()> {
    require!(new_pauser != Pubkey::default(), MinUsdError::InvalidRecipient);
    ctx.accounts.config.pauser = new_pauser;
    msg!("set_pauser {}", new_pauser);
    Ok(())
}

pub fn set_compliance(ctx: Context<AdminSet>, new_compliance: Pubkey) -> Result<()> {
    require!(new_compliance != Pubkey::default(), MinUsdError::InvalidRecipient);
    ctx.accounts.config.compliance = new_compliance;
    msg!("set_compliance {}", new_compliance);
    Ok(())
}

#[derive(Accounts)]
pub struct AdminSet<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = admin
    )]
    pub config: Account<'info, Config>,
}
