//file: src/instructions/redeem_funds.rs
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, CloseAccount, Mint, Token, TokenAccount, Transfer};
use sha2::{Digest, Sha256};

use crate::error::CustomError;
use crate::{EscrowAccount, EscrowSOLAccount, SEED_ESCROW_SOL, SEED_ESCROW_SPL};

#[derive(Accounts)]
pub struct RedeemFundsSPL<'info> {
    #[account(mut)]
    pub recipient: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED_ESCROW_SPL, escrow_account.sender.as_ref(), &escrow_account.hash_of_secret],
        bump = escrow_account.bump,
        close = sender,
    )]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut, owner = token::ID,)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = escrow_account,
        associated_token::mint = token_mint,
        associated_token::authority = recipient,
        owner = token::ID,
        
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,
    #[account(address = escrow_account.token_mint, owner = token::ID,)]
    pub token_mint: Account<'info, Mint>,
    #[account(mut, address = escrow_account.sender)]
    /// CHECK: This is safe because we check the address
    pub sender: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct RedeemFundsSOL<'info> {
    #[account(mut)]
    pub recipient: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED_ESCROW_SOL, escrow_account.sender.as_ref(), &escrow_account.hash_of_secret],
        bump = escrow_account.bump,
        close = sender,
    )]
    pub escrow_account: Account<'info, EscrowSOLAccount>,
    #[account(mut, address = escrow_account.sender)]
    /// CHECK: This is safe because we check the address
    pub sender: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

// Redeem funds for SPL tokens
pub fn redeem_funds_spl(ctx: Context<RedeemFundsSPL>, secret: String) -> Result<()> {
    //let escrow_account = &mut ctx.accounts.escrow_account;
    let provided_hash = {
        let mut hasher = Sha256::new();
        hasher.update(secret.as_bytes());
        hasher.finalize()
    };

    require!(
        provided_hash[..] == ctx.accounts.escrow_account.hash_of_secret,
        CustomError::InvalidSecret
    );

    require!(
        !ctx.accounts.escrow_account.is_redeemed,
        CustomError::AlreadyRedeemed
    );


    ctx.accounts.escrow_account.is_redeemed = true;

    // Transfer tokens to recipient
    let seeds = &[
        SEED_ESCROW_SPL,
        ctx.accounts.escrow_account.sender.as_ref(),
        &ctx.accounts.escrow_account.hash_of_secret,
        &[ctx.accounts.escrow_account.bump],
    ];
    let signer = &[&seeds[..]];
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_accounts = Transfer {
        from: ctx.accounts.escrow_token_account.to_account_info(),
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: ctx.accounts.escrow_account.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(cpi_program.clone(), cpi_accounts, signer);
    token::transfer(cpi_ctx, ctx.accounts.escrow_account.amount)?;

    // Close escrow token account
    let cpi_accounts = CloseAccount {
        account: ctx.accounts.escrow_token_account.to_account_info(),
        destination: ctx.accounts.sender.to_account_info(), // Refund rent to sender
        authority: ctx.accounts.escrow_account.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(cpi_program.clone(), cpi_accounts, signer);
    token::close_account(cpi_ctx)?;

    // Close escrow account and refund rent to sender
    let escrow_account_info = ctx.accounts.escrow_account.to_account_info();
    let sender_info = ctx.accounts.sender.to_account_info();
    **sender_info.lamports.borrow_mut() += escrow_account_info.lamports();
    **escrow_account_info.lamports.borrow_mut() = 0;

    Ok(())
}

pub fn redeem_funds_sol(ctx: Context<RedeemFundsSOL>, secret: String) -> Result<()> {
    let escrow_account = &mut ctx.accounts.escrow_account;

    // Hash the provided secret
    let provided_hash = {
        let mut hasher = Sha256::new();
        hasher.update(secret.as_bytes());
        hasher.finalize()
    };


    require!(
        provided_hash[..] == escrow_account.hash_of_secret,
        CustomError::InvalidSecret
    );


    require!(!escrow_account.is_redeemed, CustomError::AlreadyRedeemed);


    escrow_account.is_redeemed = true;

    // Transfer SOL to recipient
    let escrow_account_info = escrow_account.to_account_info();
    let recipient_info = ctx.accounts.recipient.to_account_info();

    **escrow_account_info.lamports.borrow_mut() -= escrow_account.amount;
    **recipient_info.lamports.borrow_mut() += escrow_account.amount;

    // Close escrow account and refund remaining lamports to sender
    let sender_info = ctx.accounts.sender.to_account_info();
    **sender_info.lamports.borrow_mut() += escrow_account_info.lamports();
    **escrow_account_info.lamports.borrow_mut() = 0;

    Ok(())
}
