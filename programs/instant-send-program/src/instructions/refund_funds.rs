use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    CloseAccount, Mint, TokenAccount, TokenInterface, TransferChecked,
};
use sha2::{Digest, Sha256};

use crate::error::CustomError;
use crate::{EscrowAccount, EscrowSOLAccount, SEED_ESCROW_SOL, SEED_ESCROW_SPL};

#[derive(Accounts)]
pub struct RefundFundsSPL<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED_ESCROW_SPL, &escrow_account.hash_of_secret],
        bump = escrow_account.bump,
        close = sender, // Refunds the rent-exempt balance to the sender
    )]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut)]
    pub escrow_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, address = escrow_account.sender)]
    /// CHECK: This is safe because we verify the address
    pub sender: AccountInfo<'info>,
    #[account(address = escrow_account.token_mint, mint::token_program = token_program)]
    pub token_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> RefundFundsSPL<'info> {
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

    pub fn transfer_tokens_back_to_sender(
        &self,
        amount: u64,
        signer_seeds: &[&[&[u8]]],
    ) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: self.escrow_token_account.to_account_info(),
            to: self.sender.to_account_info(),
            authority: self.escrow_account.to_account_info(),
            mint: self.token_mint.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        anchor_spl::token_interface::transfer_checked(cpi_ctx, amount, self.token_mint.decimals)
    }

    pub fn close_escrow_token_account(&self, signer_seeds: &[&[&[u8]]]) -> Result<()> {
        let cpi_accounts = CloseAccount {
            account: self.escrow_token_account.to_account_info(),
            destination: self.sender.to_account_info(),
            authority: self.escrow_account.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        anchor_spl::token_interface::close_account(cpi_ctx)
    }
}

pub fn refund_funds_spl(ctx: Context<RefundFundsSPL>, secret: String) -> Result<()> {
    ctx.accounts.verify_secret(&secret)?;

    require!(
        Clock::get()?.unix_timestamp > ctx.accounts.escrow_account.expiration_time,
        CustomError::NotExpired
    );

    let seeds = &[
        SEED_ESCROW_SPL,
        &ctx.accounts.escrow_account.hash_of_secret,
        &[ctx.accounts.escrow_account.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    ctx.accounts
        .transfer_tokens_back_to_sender(ctx.accounts.escrow_account.amount, signer_seeds)?;

    ctx.accounts.close_escrow_token_account(signer_seeds)?;

    Ok(())
}

#[derive(Accounts)]
pub struct RefundFundsSOL<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED_ESCROW_SOL, &escrow_account.hash_of_secret],
        bump = escrow_account.bump,
        close = sender, // Refunds the rent-exempt balance to the sender
    )]
    pub escrow_account: Account<'info, EscrowSOLAccount>,
    #[account(mut, address = escrow_account.sender)]
    /// CHECK: This is safe because we verify the address
    pub sender: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> RefundFundsSOL<'info> {
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

    pub fn transfer_sol_back_to_sender(&self, amount: u64) -> Result<()> {
        **self.sender.to_account_info().try_borrow_mut_lamports()? += amount;
        **self
            .escrow_account
            .to_account_info()
            .try_borrow_mut_lamports()? -= amount;
        Ok(())
    }
}

pub fn refund_funds_sol(ctx: Context<RefundFundsSOL>, secret: String) -> Result<()> {
    ctx.accounts.verify_secret(&secret)?;

    require!(
        Clock::get()?.unix_timestamp > ctx.accounts.escrow_account.expiration_time,
        CustomError::NotExpired
    );
    ctx.accounts
        .transfer_sol_back_to_sender(ctx.accounts.escrow_account.amount)?;

    Ok(())
}
