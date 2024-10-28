//file: constants
use anchor_lang::prelude::*;

#[constant]
//pub const SEED: &str = "anchor"
pub const SEED_ESCROW_SPL: &[u8] = b"escrow_spl";
pub const SEED_ESCROW_SOL: &[u8] = b"escrow_sol";
pub const ANCHOR_DISCRIMINATOR_SIZE: usize = 8;
