use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("The funds have already been redeemed.")]
    AlreadyRedeemed,
    // #[msg("The sender is invalid.")]
    // InvalidSender,
    #[msg("The transfer has not expired yet.")]
    NotExpired,
    #[msg("The secret is invalid")]
    InvalidSecret,
}
