use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::constants::SEED;
use crate::Escrow;

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = maker, // This tells Anchor: “After the instruction finishes, close this account and send its remaining lamports (rent) to the maker account.”
        has_one = mint_a, // assert!(escrow.mint_a == mint_a.key()); This tells Anchor to verify a relationship between escrow and mint_a
        has_one = maker, // assert!(escrow.maker == maker.key()); Verifies that the signer really owns the escrow and isn't spoofing someone else’s account.
        seeds = [SEED.as_bytes(), maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,

}

impl<'info> Refund<'info> {
    pub fn refund_and_close_vault(&mut self) -> Result<()> {
        let signer_seeds: [&[&[u8]]; 1] = [&[
            SEED.as_bytes(),
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];
        
        let transfer_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.maker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info()
        };

        let transfer_cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), transfer_accounts, &signer_seeds);

        transfer_checked(transfer_cpi_ctx, self.vault.amount, self.mint_a.decimals)?;

        // Close the vault account, sending rent to the maker
        let close_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let close_cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), close_accounts, &signer_seeds);

        close_account(close_cpi_ctx)
    }
}