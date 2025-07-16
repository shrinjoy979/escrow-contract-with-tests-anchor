use anchor_lang::prelude::*;

declare_id!("AWpVNRScQtfjEKPe94UeyRmj69NZKt7jkH1bquvEK12o");

#[program]
pub mod escrow_contract_with_tests_anchor {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
