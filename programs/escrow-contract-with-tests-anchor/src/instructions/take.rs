use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, token_interface::{transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface, TransferChecked, close_account}
};

use crate::constants::SEED;
use crate::state::Escrow;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut)]
    pub maker: SystemAccount<'info>, // The original escrow creator. Used to receive tokens and reclaim rent.

    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = maker, // This tells Anchor: “After the instruction finishes, close this account and send its remaining lamports (rent) to the maker account.”
        has_one = maker, // The escrow.maker field (on-chain) must match the provided maker account (in this context). This ensures the taker can't cheat by passing in a random maker.
        has_one = mint_a, // escrow.mint_a must match the mint_a account provided in the instruction
        has_one = mint_b, // escrow.mint_b must match the mint_b account
        seeds = [SEED.as_bytes(), maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump
    )]
    // These checks prevent mismatches or malicious manipulation.
    escrow: Account<'info, Escrow>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Take<'info> {
    pub fn transfer_to_maker(&mut self) -> Result<()> {

        let transfer_account = TransferChecked {
            from: self.taker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), transfer_account);
        transfer_checked(ctx, self.escrow.receive, self.mint_a.decimals)?;

        Ok(())
    }

    // Called when the taker accepts the escrow offer (swap success).
    // Transferring all tokens from the vault PDA to the taker ATA
    // Then closing the vault and sending rent to the maker

    pub fn withdraw_and_close_vault(&mut self) -> Result<()> {

        // This line is creating the "secret recipe" (seeds) used to re-generate the PDA (Program Derived Address) for the escrow account, so it can sign on behalf of itself.
        let signer_seeds: [&[&[u8]]; 1] = [&[
            SEED.as_bytes(),
            self.maker.to_owned().key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump]
        ]];

        // Prepare transfer of Token A (from escrow vault → maker)
        let accounts = TransferChecked{
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        // Execute the transfer using CPI with PDA signer
        // Transfers entire vault balance (self.vault.amount) from PDA → taker

        // CpiContext::new_with_signer tells Anchor: “The escrow PDA is signing this CPI.”
        // transfer_checked(...) calls the SPL Token TransferChecked instruction

        let ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), accounts, &signer_seeds);
        transfer_checked(ctx, self.vault.amount, self.mint_a.decimals)?;

        // Prepare to close the vault account
        let accounts = CloseAccount{
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        // Execute the vault close with PDA signer
        let ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), accounts, &signer_seeds);

        // Vault account is deleted from the chain
        close_account(ctx)
    }

}