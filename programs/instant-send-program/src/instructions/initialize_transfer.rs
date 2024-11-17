//file: src/instruction/initialize_transfer.rs
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::{
    EscrowAccount, EscrowSOLAccount, ANCHOR_DISCRIMINATOR_SIZE, SEED_ESCROW_SOL, SEED_ESCROW_SPL,
};

#[derive(Accounts)]
#[instruction(amount: u64, expiration_time: i64, hash_of_secret: [u8; 32])]
pub struct InitializeTransferSPL<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,
    #[account(init, payer = sender, space = ANCHOR_DISCRIMINATOR_SIZE + EscrowAccount::INIT_SPACE, seeds = [SEED_ESCROW_SPL, &hash_of_secret[..]], bump)]
    pub escrow_account: Account<'info, EscrowAccount>,
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

impl<'info> InitializeTransferSPL<'info> {
    pub fn into_transfer_to_escrow_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.sender_token_account.to_account_info(),
            to: self.escrow_token_account.to_account_info(),
            authority: self.sender.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    // REVIEW: should be claim_rent_from_sender
    pub fn cliam_rent_from_sender(&self) -> Result<()> {
        let rent = Rent::get()?;
        let recipient_token_account_rent = rent.minimum_balance(TokenAccount::LEN);

        anchor_lang::system_program::transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: self.sender.to_account_info(),
                    to: self.escrow_account.to_account_info(),
                },
            ),
            recipient_token_account_rent,
        )?;
        Ok(())
    }
}

pub fn initialize_transfer_spl(
    ctx: Context<InitializeTransferSPL>,
    amount: u64,
    expiration_time: i64,
    hash_of_secret: [u8; 32],
) -> Result<()> {
    let escrow_account = &mut ctx.accounts.escrow_account;

    escrow_account.sender = *ctx.accounts.sender.key;
    escrow_account.amount = amount;
    escrow_account.expiration_time = expiration_time;
    escrow_account.is_redeemed = false;
    escrow_account.token_mint = ctx.accounts.token_mint.key();
    escrow_account.hash_of_secret = hash_of_secret;
    escrow_account.bump = ctx.bumps.escrow_account;

    token::transfer(ctx.accounts.into_transfer_to_escrow_context(), amount)?;

    //if you for some reason want to claim the rent for creating a token account for the reciever
    //ctx.accounts.claim_rent_from_sender()?;

    Ok(())
}

// Account structures for SOL
#[derive(Accounts)]
#[instruction(amount: u64, expiration_time: i64, hash_of_secret: [u8; 32])]
pub struct InitializeTransferSOL<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,
    #[account(init, payer = sender, space = ANCHOR_DISCRIMINATOR_SIZE + EscrowSOLAccount::INIT_SPACE, seeds = [SEED_ESCROW_SOL, &hash_of_secret[..]], bump)]
    pub escrow_account: Account<'info, EscrowSOLAccount>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitializeTransferSOL<'info> {
    pub fn into_transfer_sol_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, anchor_lang::system_program::Transfer<'info>> {
        let transfer_instruction = anchor_lang::system_program::Transfer {
            from: self.sender.to_account_info(),
            to: self.escrow_account.to_account_info(),
        };
        CpiContext::new(self.system_program.to_account_info(), transfer_instruction)
    }
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

    anchor_lang::system_program::transfer(ctx.accounts.into_transfer_sol_context(), amount)?;

    Ok(())
}
