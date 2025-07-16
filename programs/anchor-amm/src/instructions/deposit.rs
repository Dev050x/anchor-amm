#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use anchor_lang::{prelude::*};
use anchor_spl::{associated_token::AssociatedToken, token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer}};
use constant_product_curve::ConstantProduct;

use crate::Config;
use crate::error::AmmError;

#[derive(Accounts)]

pub struct Deposit<'info>{
    #[account(mut)]
    pub user:Signer<'info>,
    pub mint_x:Account<'info , Mint>,
    pub mint_y:Account<'info , Mint>,
    #[account(
        mut,
        has_one = mint_x,
        has_one = mint_y,
        seeds = [b"config",config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config:Account<'info , Config>,

    #[account(  
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config
    )]
    pub vault_x:Account<'info , TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config
    )]
    pub vault_y:Account<'info , TokenAccount>,  

    #[account(
        mut,
        seeds = [b"lp" , config.key().as_ref()],
        bump = config.lp_bum,
        mint::decimals = 6,
        mint::authority = config
    )]
    pub mint_lp:Account<'info , Mint>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
    )]
    pub user_lp:Account<'info , TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_x , 
        associated_token::authority = user
    )]
    pub user_x:Account<'info , TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y , 
        associated_token::authority = user
    )]
    pub user_y:Account<'info , TokenAccount>,
    

    pub system_program:Program<'info , System>,
    pub token_program:Program<'info , Token>,
    pub associated_token_program: Program<'info , AssociatedToken>
}


impl<'info> Deposit<'info>{

    pub fn deposit(&mut self , amount: u64 , max_x:u64 , max_y:u64) -> Result<()> {

        require!(amount != 0 , AmmError::InvalidAmount);
        require!(self.config.locked == false , AmmError::PoolLocked);

        let (x , y) = match self.mint_lp.supply == 0 && self.vault_x.amount == 0 && self.vault_y.amount == 0 {
            true => (max_x , max_y),
            false => {
                let amount = ConstantProduct::xy_deposit_amounts_from_l(
                    self.vault_x.amount,
                    self.vault_y.amount,
                    self.mint_lp.supply,
                    amount,
                    6
                ).unwrap();
                (amount.x , amount.y)
            }
        };
        require!(x <= max_x , AmmError::SlippageExceeded);
        require!(y <= max_y , AmmError::SlippageExceeded);

        self.deposit_token(x, true);
        self.deposit_token(y, false);

        self.mint_lp(amount);

        Ok(())
    }


    pub fn deposit_token(&mut self , amount: u64 , is_x:bool ) -> Result<()> {

        let (from , to ) = match is_x {
            true => (self.user_x.to_account_info() , self.vault_x.to_account_info()),
            false => (self.user_y.to_account_info() , self.vault_y.to_account_info())
        };

        let cpiContext = CpiContext::new(self.token_program.to_account_info(), Transfer{
            from,
            to,
            authority:self.user.to_account_info()
        });

        transfer(cpiContext,amount)?;

        Ok(())
    }

    pub fn mint_lp(&mut self, amount:u64) -> Result<()> {
        
        let seeds = &[
            &b"config"[..],
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ];

        let signer_seeds = &[&seeds[..]];


        let cpiContext = CpiContext::new_with_signer(self.token_program.to_account_info() , MintTo{
            mint:self.mint_lp.to_account_info(),
            to:self.user_lp.to_account_info(),
            authority:self.config.to_account_info(),
        },signer_seeds);

        mint_to(cpiContext, amount)?;
        Ok(())
    }

}