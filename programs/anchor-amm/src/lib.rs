#![allow(unexpected_cfgs)]
#![allow(deprecated)]
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("HEnA8ZWiYaEKWxFB3f5bS3yTJgyibT6QHR2ugsNfS3tx");

#[program]
pub mod anchor_amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize> , seeds: u64 , authority:Option<Pubkey> , fee:u16) -> Result<()> {
        ctx.accounts.initialize(seeds, &ctx.bumps, authority, fee)?;
        Ok(())
    }
    pub fn deposit(ctx: Context<Deposit> , amount:u64 , max_x:u64 , max_y:u64 ) -> Result<()>  {
        ctx.accounts.deposit(amount, max_x, max_y)?;
        Ok(())
    }
    pub fn withdraw(ctx: Context<Withdraw> , amount:u64 , max_x:u64 , max_y:u64 ) -> Result<()>  {
        ctx.accounts.withdraw(amount, max_x, max_y)?;
        ctx.accounts.burn_token(amount)?;
        Ok(())
    }
}

