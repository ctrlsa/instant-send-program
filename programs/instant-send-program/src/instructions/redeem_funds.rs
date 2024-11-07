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
        close = recipient, //we return the rent-exempt balance to the reciever, such that have enough sol, to make an inital transaction with their spl tokens.
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

impl<'info> RedeemFundsSPL<'info> {
    pub fn verify_secret(&self, secret: &str) -> Result<()> {
        let provided_hash = {
            let mut hasher = Sha256::new();
            hasher.update(secret.as_bytes());
            hasher.finalize()
        };
        require!(
            provided_hash[..] == self.escrow_account.hash_of_secret,
            CustomError::InvalidSecret
        );
        Ok(())
    }

    pub fn transfer_tokens_to_recipient(&self, amount: u64, signer: &[&[&[u8]]]) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.escrow_token_account.to_account_info(),
            to: self.recipient_token_account.to_account_info(),
            authority: self.escrow_account.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)
    }

    pub fn close_escrow_token_account(&self, signer: &[&[&[u8]]]) -> Result<()> {
        let cpi_accounts = CloseAccount {
            account: self.escrow_token_account.to_account_info(),
            destination: self.signer.to_account_info(), // Refunds the rent to the signer
            authority: self.escrow_account.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::close_account(cpi_ctx)
    }


}

// Redeem funds for SPL tokens
pub fn redeem_funds_spl(ctx: Context<RedeemFundsSPL>, secret: String) -> Result<()> {
    ctx.accounts.verify_secret(&secret)?;
    require!(
        !ctx.accounts.escrow_account.is_redeemed,
        CustomError::AlreadyRedeemed
    );
    ctx.accounts.escrow_account.is_redeemed = true;

    // Transfer tokens to recipient
    let seeds = &[
        SEED_ESCROW_SPL,
        &ctx.accounts.escrow_account.hash_of_secret,
        &[ctx.accounts.escrow_account.bump],
    ];
    let signer = &[&seeds[..]];
    ctx.accounts.transfer_tokens_to_recipient(ctx.accounts.escrow_account.amount, signer)?;
    ctx.accounts.close_escrow_token_account(signer)?;


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
        close = sender,
    )]
    pub escrow_account: Account<'info, EscrowSOLAccount>,
    /// CHECK: The sender account is provided by the caller and verified in the program logic.
    #[account(mut, address = escrow_account.sender)]
    pub sender: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> RedeemFundsSOL<'info> {
    pub fn verify_secret(&self, secret: &str) -> Result<()> {
        let provided_hash = {
            let mut hasher = Sha256::new();
            hasher.update(secret.as_bytes());
            hasher.finalize()
        };
        require!(
            provided_hash[..] == self.escrow_account.hash_of_secret,
            CustomError::InvalidSecret
        );
        Ok(())
    }

    pub fn transfer_sol_to_recipient(&self, amount: u64) -> Result<()> {
        **self.recipient.to_account_info().try_borrow_mut_lamports()? += amount;
        **self.escrow_account.to_account_info().try_borrow_mut_lamports()? -= amount;
        Ok(())
    }

    // pub fn refund_remaining_lamports_to_sender(&self) -> Result<()> {
    //     let remaining_lamports = **self.escrow_account.to_account_info().lamports.borrow();
    //     if remaining_lamports > 0 {
    //         **self.sender.to_account_info().try_borrow_mut_lamports()? += remaining_lamports;
    //         **self.escrow_account.to_account_info().lamports.borrow_mut() = 0;
    //     }
    //     Ok(())
    // }
}


pub fn redeem_funds_sol(ctx: Context<RedeemFundsSOL>, secret: String) -> Result<()> {
    ctx.accounts.verify_secret(&secret)?;

    require!(
        !ctx.accounts.escrow_account.is_redeemed,
        CustomError::AlreadyRedeemed
    );

    ctx.accounts.escrow_account.is_redeemed = true;
    ctx.accounts.transfer_sol_to_recipient(ctx.accounts.escrow_account.amount)?;
    // ctx.accounts.refund_remaining_lamports_to_sender()?;

    Ok(())
}