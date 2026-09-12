use anchor_lang::prelude::*;

use crate::state::Config;
use crate::CONFIG_SEED;

pub fn pause(ctx: Context<SetPaused>) -> Result<()> {
    ctx.accounts.config.paused = true;
    emit!(Paused {
        pauser: ctx.accounts.pauser.key(),
    });
    msg!("paused by {}", ctx.accounts.pauser.key());
    Ok(())
}

pub fn unpause(ctx: Context<SetPaused>) -> Result<()> {
    ctx.accounts.config.paused = false;
    emit!(Unpaused {
        pauser: ctx.accounts.pauser.key(),
    });
    msg!("unpaused by {}", ctx.accounts.pauser.key());
    Ok(())
}

#[event]
pub struct Paused {
    pub pauser: Pubkey,
}

#[event]
pub struct Unpaused {
    pub pauser: Pubkey,
}

#[derive(Accounts)]
pub struct SetPaused<'info> {
    pub pauser: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = pauser
    )]
    pub config: Account<'info, Config>,
}
