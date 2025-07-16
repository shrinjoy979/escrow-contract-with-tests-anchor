#![allow(unexpected_cfgs, deprecated)]
use anchor_lang::prelude::*;

pub mod constants;
pub mod state;
pub mod instructions;

use instructions::*;
use state::*;

declare_id!("H9jeCtJaWeqtpCeFcoXwAfkbtkoPYpFRw64PvHreyVwZ");

// The #[program] macro tells Anchor that this is the entry point module containing all your instruction handlers (like make, take, etc).
#[program]
pub mod escrow_contract_with_tests {
    // Brings everything from the outer lib.rs module into the #[program] scope. This includes your use instructions::*, so you can use Make, Take, Refund directly.
    use super::*;

    /*  make is the instruction to initialize the escrow account and vault.
        Context<Make> contains all the accounts needed (like maker, escrow, mint_a, vault, etc).
        seed is a unique identifier to distinguish multiple escrows from the same maker.
        receive is the amount of token B (that the taker must deposit to claim token A).
        ctx.accounts.init_esrow(...) is a helper method (defined in Make impl block) that initializes the Escrow struct.
        ctx.bumps gives access to auto-calculated PDA bumps.
    */

    pub fn make(ctx: Context<Make>, seed: u64, receive: u64, deposit: u64) -> Result<()> {
        ctx.accounts.init_esrow(seed, receive, &ctx.bumps)?;
        ctx.accounts.deposit(deposit)?;
        Ok(())
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.transfer_to_maker()?;
        ctx.accounts.withdraw_and_close_vault()
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }
}
