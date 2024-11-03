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
    pub signer: Signer<'info>,
    /// CHECK: The recipient account is provided by the caller and verified in the program logic.
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [SEED_ESCROW_SPL, &escrow_account.hash_of_secret],
        bump = escrow_account.bump,
        close = sender,
    )]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut, owner = token::ID,)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = signer,
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

    msg!("Recipient SOL Balance Before Transfer: {}", ctx.accounts.recipient.to_account_info().lamports());
    msg!("Recipient SPL Token Balance Before Transfer: {}", ctx.accounts.recipient_token_account.amount);
    // Transfer tokens to recipient
    let seeds = &[
        SEED_ESCROW_SPL,
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

    
    msg!("Recipient SOL Balance After Transfer: {}", ctx.accounts.recipient.to_account_info().lamports());
    msg!("Recipient SPL Token Balance After Transfer: {}", ctx.accounts.recipient_token_account.amount);

    // Close escrow token account
    let cpi_accounts = CloseAccount {
        account: ctx.accounts.escrow_token_account.to_account_info(),
        destination: ctx.accounts.signer.to_account_info(), // Refund rent to the signer, for paying for it
        authority: ctx.accounts.escrow_account.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(cpi_program.clone(), cpi_accounts, signer);
    token::close_account(cpi_ctx)?;

    
    // let remaining_lamports = **ctx.accounts.escrow_account.to_account_info().lamports.borrow();
    // **ctx.accounts.sender.to_account_info().try_borrow_mut_lamports()? += remaining_lamports;
    // **ctx.accounts.escrow_account.to_account_info().lamports.borrow_mut() = 0;

    Ok(())
}

#[derive(Accounts)]
pub struct RedeemFundsSOL<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    /// CHECK: The recipient account is provided by the caller and verified in the program logic.
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [SEED_ESCROW_SOL, &escrow_account.hash_of_secret],
        bump = escrow_account.bump,
        
    )]
    pub escrow_account: Account<'info, EscrowSOLAccount>,
    /// CHECK: The sender account is provided by the caller and verified in the program logic.
    #[account(mut, address = escrow_account.sender)]
    pub sender: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

pub fn redeem_funds_sol(ctx: Context<RedeemFundsSOL>, secret: String) -> Result<()> {
    let escrow_account = &mut ctx.accounts.escrow_account;
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

    **ctx.accounts.recipient.to_account_info().try_borrow_mut_lamports()? += escrow_account.amount;
    **ctx.accounts.escrow_account.to_account_info().try_borrow_mut_lamports()? -= escrow_account.amount;

    let remaining_lamports = **ctx.accounts.escrow_account.to_account_info().lamports.borrow();
    **ctx.accounts.sender.to_account_info().try_borrow_mut_lamports()? += remaining_lamports;
    **ctx.accounts.escrow_account.to_account_info().lamports.borrow_mut() = 0;

    Ok(())
}