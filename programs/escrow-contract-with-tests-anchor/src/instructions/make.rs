use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{AssociatedToken},
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked}
};

use crate::constants::*;
use crate::Escrow;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>, // Signer is a type in Anchor.

    // it defines a token mint account (mint_a) with a constraint.
    // mint::token_program = token_program. This is a constraint specific to a Mint account.
    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut, // since, maker is depositing it, he already have the ata for these token
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    // The escrow (PDA)
    // b"escrow" => fixed size array of raw bytes &[u8; 6]
    // .as_ref() converts it to a byte slice (&[u8]) for use in PDA seeds
    // seed is a u64 passed in by the user
    // .to_le_bytes() converts it to a [u8; 8] array in little-endian
    #[account(
        init,
        payer = maker,
        seeds = [SEED.as_bytes(), maker.key().as_ref(), seed.to_le_bytes().as_ref()], // for safty, we are associating with maker key | seed will get from client side
        space = ANCHOR_DISCREMINATOR + Escrow::INIT_SPACE,
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    // The vault ATA for mint_a, owned by the escrow PDA.
    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    // token_program: The name of the account in your instruction context.
    // Interface: Anchor's wrapper that supports different token program interfaces.
    /*
        Why use Interface instead of Program?
        Interface<'info, TokenInterface>
        
        Allows support for both:
        Original SPL Token Program
        Newer Token-2022 Program (more features)
    */
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Make<'info> {
    // self refers to the Make struct. That struct holds all your accounts (like maker, mint_a, escrow, etc.), so you can access them as self.maker, self.escrow, and so on.
    // set_inner(...) replaces the uninitialized state with the actual data.
    // MakeBumps beacuse we are inside Make

    pub fn init_esrow(&mut self, seed: u64, receive: u64, bumps: &MakeBumps) -> Result<()> {
        self.escrow.set_inner(
            Escrow { 
                seed, 
                maker: self.maker.key(), 
                mint_a: self.mint_a.key(), 
                mint_b: self.mint_b.key(), 
                receive, 
                bump: bumps.escrow 
            });
        Ok(())
    }

    // TransferChecked is a struct
    // maker’s token account into the vault
    // to_account_info returns AccountInfo struct
    // Only the maker needs to sign; no PDA signer_seeds are required for deposit.
    pub fn deposit(&mut self, deposit: u64) -> Result<()> {
        let transfer_accounts = TransferChecked {
            from: self.maker_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info(),
        };

        // You’re creating a CPI context that tells Anchor: “I want to call the SPL Token Program with these accounts and settings.”
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

        transfer_checked(cpi_ctx, deposit, self.mint_a.decimals)?;

        Ok(())
    }
}