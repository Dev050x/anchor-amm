use anchor_lang::{prelude::*};
use anchor_spl::{token::{Mint , TokenAccount , Token} , associated_token::{AssociatedToken}};

use crate::Config;

#[derive(Accounts)]
#[instruction(seed:u64)]
pub struct Initialize<'info>{
    #[account(mut)]
    pub initializer:Signer<'info>,
    pub mint_x:Account<'info , Mint>,
    pub mint_y:Account<'info , Mint>,
    #[account(
        init,
        seeds = [b"config",seed.to_le_bytes().as_ref()],
        bump,
        payer = initializer,
        space = 8 + Config::INIT_SPACE,
    )]
    pub config:Account<'info , Config>,

    #[account(
        init,
        payer = initializer,
        associated_token::mint = mint_x,
        associated_token::authority = config
    )]
    pub vault_x:Account<'info , TokenAccount>,
    #[account(
        init,
        payer = initializer,
        associated_token::mint = mint_y,
        associated_token::authority = config
    )]
    pub vault_y:Account<'info , TokenAccount>,  

    #[account(
        init,
        payer = initializer,
        seeds = [b"lp" , config.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = config
    )]
    pub mint_lp:Account<'info , Mint>,

    pub system_program:Program<'info , System>,
    pub token_program:Program<'info , Token>,
    pub associated_token_program: Program<'info , AssociatedToken>
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self , seed:u64 , bump:&InitializeBumps , authority:Option<Pubkey> , fee:u16) -> Result<()> {
        self.config.set_inner(
            Config { 
                seed,
                mint_x: self.mint_x.key(),
                mint_y: self.mint_y.key(),
                authority,
                fee,
                config_bump: bump.config,
                lp_bum: bump.mint_lp,
                locked: false
            }
        );

        Ok(())
    }
}