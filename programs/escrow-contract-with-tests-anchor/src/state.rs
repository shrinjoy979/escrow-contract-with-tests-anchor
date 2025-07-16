use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)] // This macro automatically calculates how much space your struct needs when stored in a Solana account.
pub struct Escrow {
    pub seed: u64, // A unique number chosen by maker to differentiate multiple escrows.
    pub maker: Pubkey, // The public key of the person who created the escrow
    pub mint_a: Pubkey, // Token mint that the maker is offering
    pub mint_b: Pubkey, // Token mint that the maker wants to receive
    pub receive: u64, // Amount of token B (mint_b) that the taker must send to accept the deal.
    pub bump: u8, // PDA bump used to derive the vault authority PDA safely.
}