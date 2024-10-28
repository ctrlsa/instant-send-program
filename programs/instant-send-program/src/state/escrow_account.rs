//file: src/state/escrow_account.rs
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct EscrowAccount {
    pub sender: Pubkey,
    pub amount: u64,
    pub expiration_time: i64,
    pub is_redeemed: bool,
    pub token_mint: Pubkey,
    pub hash_of_secret: [u8; 32],
    pub bump: u8,
}
