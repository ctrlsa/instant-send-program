//file: src/instruction/initialize_transfer.rs
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::{
    EscrowAccount, EscrowSOLAccount, ANCHOR_DISCRIMINATOR_SIZE, SEED_ESCROW_SOL, SEED_ESCROW_SPL,
};

#[derive(Accounts)]
#[instruction(hash_of_secret: [u8; 32])]
pub struct InitializeTransferSPL<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,
    #[account(init, payer = sender, space = ANCHOR_DISCRIMINATOR_SIZE + EscrowAccount::INIT_SPACE, seeds = [SEED_ESCROW_SPL, sender.key().as_ref(), &hash_of_secret], bump)]
    pub escrow_account: Account<'info, EscrowAccount>,
    // hash_of_secret: [u8; 32], unique_seed(public key) could be added here
    #[account(
        init,
        payer = sender,
        associated_token::mint = token_mint,
        associated_token::authority = escrow_account,
        owner = token::ID,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(mut, owner = token::ID)]
    pub sender_token_account: Account<'info, TokenAccount>,
    #[account(owner = token::ID)]
    pub token_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

// Account structures for SOL
#[derive(Accounts)]
#[instruction(hash_of_secret: [u8; 32])]
pub struct InitializeTransferSOL<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,
    #[account(init, payer = sender, space = ANCHOR_DISCRIMINATOR_SIZE + EscrowSOLAccount::INIT_SPACE, seeds = [SEED_ESCROW_SOL, sender.key().as_ref(), &hash_of_secret], bump)]
    pub escrow_account: Account<'info, EscrowSOLAccount>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn initialize_transfer_spl(
    ctx: Context<InitializeTransferSPL>,
    amount: u64,
    expiration_time: i64, // Use i64 for Unix timestamp
    hash_of_secret: [u8; 32],
) -> Result<()> {
    let escrow_account = &mut ctx.accounts.escrow_account;

    // Initialize escrow account data
    escrow_account.sender = *ctx.accounts.sender.key;
    escrow_account.amount = amount;
    escrow_account.expiration_time = expiration_time;
    escrow_account.is_redeemed = false;
    escrow_account.token_mint = ctx.accounts.token_mint.key();
    escrow_account.hash_of_secret = hash_of_secret;
    escrow_account.bump = ctx.bumps.escrow_account;

    let rent = Rent::get()?;
    let recipient_token_account_rent = rent.minimum_balance(TokenAccount::LEN);

    // Transfer tokens from sender to escrow token account
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_accounts = Transfer {
        from: ctx.accounts.sender_token_account.to_account_info(),
        to: ctx.accounts.escrow_token_account.to_account_info(),
        authority: ctx.accounts.sender.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    let total_lamports = recipient_token_account_rent;
    **ctx.accounts.sender.to_account_info().lamports.borrow_mut() -= total_lamports;
    **ctx
        .accounts
        .escrow_account
        .to_account_info()
        .lamports
        .borrow_mut() += total_lamports;

    Ok(())
}

pub fn initialize_transfer_sol(
    ctx: Context<InitializeTransferSOL>,
    amount: u64,
    expiration_time: i64,
    hash_of_secret: [u8; 32],
) -> Result<()> {
    let escrow_account = &mut ctx.accounts.escrow_account;

    escrow_account.sender = *ctx.accounts.sender.key;
    escrow_account.amount = amount;
    escrow_account.expiration_time = expiration_time;
    escrow_account.is_redeemed = false;
    escrow_account.hash_of_secret = hash_of_secret;
    escrow_account.bump = ctx.bumps.escrow_account;

    // Transfer SOL to the escrow account
    **escrow_account.to_account_info().lamports.borrow_mut() += amount;
    **ctx.accounts.sender.to_account_info().lamports.borrow_mut() -= amount;

    Ok(())
}
