use anchor_lang::prelude::*;
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

pub use constants::*;
use instructions::*;
pub use state::*;
//declare_id!("4khKXMz3ttSaoxuwJ6nB93SB2PSjvj3FZP4E1gCPGHKW");
declare_id!("BCLTR5fuCWrMUWc75yKnG35mtrvXt6t2eLuPwCXA93oY");

#[program]
pub mod instant_send_program {
    use super::*;

    // SPL Token functions
    pub fn initialize_transfer_spl(
        ctx: Context<InitializeTransferSPL>,
        amount: u64,
        expiration_time: i64,
        hash_of_secret: [u8; 32],
    ) -> Result<()> {
        instructions::initialize_transfer::initialize_transfer_spl(
            ctx,
            amount,
            expiration_time,
            hash_of_secret,
        )
    }

    pub fn redeem_funds_spl(ctx: Context<RedeemFundsSPL>, secret: String) -> Result<()> {
        instructions::redeem_funds::redeem_funds_spl(ctx, secret)
    }

    pub fn initialize_transfer_sol(
        ctx: Context<InitializeTransferSOL>,
        amount: u64,
        expiration_time: i64,
        hash_of_secret: [u8; 32],
    ) -> Result<()> {
        instructions::initialize_transfer::initialize_transfer_sol(
            ctx,
            amount,
            expiration_time,
            hash_of_secret,
        )
    }

    pub fn redeem_funds_sol(ctx: Context<RedeemFundsSOL>, secret: String) -> Result<()> {
        instructions::redeem_funds::redeem_funds_sol(ctx, secret)
    }

    // pub fn refund_funds_sol(ctx: Context<RefundFundsSOL>) -> Result<()> {
    //     instructions::refund_funds::refund_funds_sol(ctx)
    // }
}
