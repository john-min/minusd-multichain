use anchor_lang::prelude::*;

#[error_code]
pub enum MinUsdError {
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Invalid recipient or account")]
    InvalidRecipient,
    #[msg("Acquire and redeem are paused")]
    Paused,
    #[msg("Account is frozen")]
    AccountIsFrozen,
    #[msg("Account is already frozen")]
    AlreadyFrozen,
    #[msg("Account is not frozen")]
    NotFrozen,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Mint decimals must be 6")]
    DecimalMismatch,
}
