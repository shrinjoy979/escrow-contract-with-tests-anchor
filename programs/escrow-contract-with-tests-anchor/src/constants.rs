use anchor_lang::prelude::*;

#[constant]
pub const SEED: &str = "escrow";
pub const ANCHOR_DISCREMINATOR: usize = 8;

/*
    What is #[constant]?
    #[constant] is an Anchor attribute macro used to mark a global constant so it becomes part of the IDL (Interface Definition Language).
    if, we don't use #[constant] => That constant exists only in Rust. It’s not exposed to clients (TypeScript/JavaScript/SDKs).
    if, we add #[constant] => Then Anchor includes SEED in the generated IDL, which makes it accessible to frontend clients.


    Why we are using &str, not String?
    Seeds must be byte slices at compile time:
    seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()]
    b"escrow" is equivalent to &[u8; 6], which comes from a &str.

    String requires heap allocation, which:
    Isn’t available in BPF (Solana runtime doesn’t support heap like normal systems).
    Would make it impossible to use as a compile-time constant.

    &str compiles to a fixed-size read-only memory segment, so it’s:
    Faster
    Safer
    More deterministic
*/
