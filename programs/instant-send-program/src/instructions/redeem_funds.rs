//file: src/instructions/redeem_funds.rs
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    CloseAccount, Mint, TokenAccount, TokenInterface, TransferChecked,
};
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
    #[account(mut)]
    pub escrow_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = token_mint,
        associated_token::authority = recipient,
        associated_token::token_program = token_program
    )]
    pub recipient_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(address = escrow_account.token_mint, mint::token_program = token_program)]
    pub token_mint: InterfaceAccount<'info, Mint>,
    #[account(mut, address = escrow_account.sender)]
    /// CHECK: This is safe because we check the address
    pub sender: AccountInfo<'info>,
    pub token_program: Interface<'info, TokenInterface>,
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

    pub fn transfer_tokens_to_recipient(
        &self,
        amount: u64,
        signer_seeds: &[&[&[u8]]],
    ) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: self.escrow_token_account.to_account_info(),
            to: self.recipient_token_account.to_account_info(),
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
            destination: self.signer.to_account_info(),
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
    let signer_seeds = &[&seeds[..]];
    ctx.accounts
        .transfer_tokens_to_recipient(ctx.accounts.escrow_account.amount, signer_seeds)?;
    ctx.accounts.close_escrow_token_account(signer_seeds)?;

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

    //Transfer: `from` must not carry data
    //occurs because the system program's transfer instruction requires that the from account must not carry data (i.e., it must be a system account with no data). In your case, the escrow_account has data associated with it (EscrowSOLAccount), so using the system program's transfer instruction is invalid.
    //In the context of transferring lamports from a program-owned account with data, you cannot use the system program's transfer instruction. Instead, you need to adjust the lamports balances directly within your program.

    // pub fn transfer_sol_to_recipient(&self, amount: u64, signer_seeds: &[&[&[u8]]]) -> Result<()> {
    //     let transfer_instruction = anchor_lang::system_program::Transfer {
    //         from: self.escrow_account.to_account_info(),
    //         to: self.recipient.to_account_info(),
    //     };
    //     let cpi_ctx = CpiContext::new_with_signer(
    //         self.system_program.to_account_info(),
    //         transfer_instruction,
    //         signer_seeds,
    //     );
    //     anchor_lang::system_program::transfer(cpi_ctx, amount)
    // }

    pub fn transfer_sol_to_recipient(&self, amount: u64) -> Result<()> {
        **self.recipient.to_account_info().try_borrow_mut_lamports()? += amount;
        **self
            .escrow_account
            .to_account_info()
            .try_borrow_mut_lamports()? -= amount;
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
    ctx.accounts
        .transfer_sol_to_recipient(ctx.accounts.escrow_account.amount)?;
    // ctx.accounts.refund_remaining_lamports_to_sender()?;

    Ok(())
}
