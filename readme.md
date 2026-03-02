Escrow Contract with Tests (Anchor)
A decentralized Escrow smart contract built on the Solana blockchain using the Anchor Framework.
This project demonstrates secure token escrow flows with complete TypeScript test coverage.

🚀 Project Overview
This program enables two parties to securely exchange SPL tokens using an on-chain escrow mechanism.

The contract supports:
Creating an escrow offer (make)
Depositing tokens into escrow (deposit)
Accepting and completing the trade (take)
Refunding tokens if the trade is cancelled (refund)
All logic is executed on-chain with Program Derived Addresses (PDAs) for secure authority control.

🛠 Tech Stack
Solana
Anchor Framework
Rust (Smart Contract)
TypeScript (Tests)
SPL Token Program
Associated Token Accounts (ATAs)

📦 Features
1️⃣ Make Escrow
Initializer creates an escrow account (PDA)
Defines:
Token mint A (offered token)
Token mint B (requested token)
Amounts for trade
Escrow PDA becomes authority

2️⃣ Deposit Tokens
Transfers offered tokens into escrow vault (ATA owned by PDA)
Uses CPI to SPL Token program

3️⃣ Take (Accept Trade)
Taker sends requested tokens to initializer
Escrow releases offered tokens to taker
Vault account is closed

4️⃣ Refund
Initializer can reclaim deposited tokens
Escrow vault is closed safely



/////////////////////// state.rs ///////////////////
Why we put Escrow in state.rs?
We put Escrow in state.rs to separate data structures from logic, making the code modular and clean.
It defines persistent on-chain data, so it's treated as program state.

in state.rs, we have define struct, then why we use #[accounts]?
We write #[account] on the struct because it's not just a normal struct — it's a Solana account struct managed by Anchor.
This tells Anchor: “This struct should be stored in a real Solana account on-chain.”
#[account] marks a struct as on-chain state so Anchor can manage its storage, validation, and deserialization automatically.

/////////////////////// mod.rs ///////////////////
What is mod.rs?
mod.rs is a module declaration file used in Rust when a folder is used as a module.
We use mod.rs to group and re-export all instruction modules (make.rs, take.rs) in one place.

What is instructions?
Instructions are the entry points or functions that a Solana program exposes for users to call (e.g., make(), take()).
They define what the program can do and the accounts required to perform those actions.

In mod.rs, we are using pub mod make; and pub use make::*; in two lines? can't we combine this to one line?
No, you cannot combine.
pub mod make; → declares a module (make.rs)
pub use make::*; → re-exports items from that module
These are two different operations:
mod brings the module into scope (compiles the file)
use makes its contents available to outer scopes

/////////////////////// make.rs ///////////////////
#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
}
here, why we are using #[derive(Accounts)]?

You use #[derive(Accounts)] on Make<'info> so Anchor knows this struct represents the set of accounts required for the make instruction.
Anchor will:
Validate the incoming accounts (PDA seeds, ownership, signer, etc.)
Deserialize each Solana account into Rust types (like Account, InterfaceAccount, Signer, etc.)
Enforce constraints (like mut, init, associated_token, etc.)

Why we use #[instruction(seed: u64)] in make.rs?
means seed is passed into the context for PDA generation.
in the Make struct, you're using seed to derive a PDA:
#[account(
    init,
    payer = maker,
    seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
    bump,
)]
pub escrow: Account<'info, Escrow>,
So Anchor must know the seed before validating this account.

Why we need escrow and valut account in make.rs?
Escrow (PDA) ==> Stores metadata about the trade (who, what, how much)
Vault (ATA) ==> Holds the actual tokens (Token A) until the trade is completed

Explain &self.escrow.seed.to_le_bytes()[..]
1. to_le_bytes()
A Rust method that converts a number (like u64) into an array of 8 bytes in little-endian format.
Why needed?
PDAs in Solana are derived from byte seeds, so we must convert all numeric values to byte arrays. Endianness matters: little-endian means the least significant byte comes first.
Example:
let seed: u64 = 42;
let bytes = seed.to_le_bytes(); 
// bytes = [42, 0, 0, 0, 0, 0, 0, 0]
So to_le_bytes() gives you: [u8; 8]
2. [..]
📌 What it is:
Rust's slice syntax.  Converts a fixed-size array into a slice (&[u8]).
Why?
Many Solana APIs (like Pubkey::find_program_address or CpiContext::new_with_signer) expect slices, not fixed arrays.
So you can't pass [u8; 8], you need &[u8].
Example:
let a: [u8; 3] = [1, 2, 3];
let slice = &a[..]; // slice is of type &[u8], not [u8; 3]
3. &
The reference operator in Rust. Turns a value into a reference to that value.

| Part            | What it does                     | Result                      |
| --------------- | -------------------------------- | --------------------------- |
| `to_le_bytes()` | Converts `u64` → `[u8; 8]`       | `[42, 0, 0, 0, 0, 0, 0, 0]` |
| `[..]`          | Takes a slice of the whole array | `&[u8]`                     |
| `&`             | Borrows the slice                | `&[u8]`                     |
