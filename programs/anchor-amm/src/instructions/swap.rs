#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount, Transfer , transfer}};
use constant_product_curve::{ConstantProduct, LiquidityPair};

use crate::Config;
use crate::error::AmmError;


#[derive(Accounts)]
pub struct Swap<'info>{
    #[account(mut)]
    pub user:Signer<'info>,
    pub mint_x:Account<'info , Mint>,
    pub mint_y:Account<'info , Mint>,
    #[account(
        mut,
        has_one=mint_x,
        has_one=mint_y,
        seeds = [b"config",config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    pub config :Account<'info , Config>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config
    )]
    pub vault_x: Account<'info , TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,    
        associated_token::authority = config
    )]
    pub vault_y: Account<'info , TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_y,
        associated_token::authority = user,
    )]
    pub user_y : Account<'info , TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_x , 
        associated_token::authority = user
    )]
    pub user_x : Account<'info , TokenAccount>,

    #[account(
        mut,
        seeds = [b"lp" , config.key().as_ref()],
        bump = config.lp_bum,
        mint::decimals = 6,
        mint::authority = config
    )]
    pub mint_lp:Account<'info , Mint>,

    pub system_program: Program<'info , System>,
    pub token_program: Program<'info , Token>,
    pub associated_token_program: Program<'info , AssociatedToken>

}

impl<'info> Swap<'info>{
    pub fn swap(&mut self , is_x:bool , min:u64 , amount:u64) -> Result<()> {

        require!(amount != 0 , AmmError::InvalidAmount);
        require!(self.config.locked == false , AmmError::PoolLocked);

        let mut curve  = ConstantProduct::init(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply,
            self.config.fee,
            None,
        ).map_err(AmmError::from)?;

        let p = match is_x {
            true => LiquidityPair::X,
            false => LiquidityPair::Y
        };

        let res = curve.swap(
            p,
            amount,
            min
        ).map_err(AmmError::from)?;

        require!(res.deposit != 0 , AmmError::InvalidAmount);
        require!(res.withdraw != 0 , AmmError::InvalidAmount);
        self.deposit_token(res.deposit, is_x)?;
        self.withdraw_token(res.withdraw, is_x)?;

        Ok(())
    }

    pub fn deposit_token(&mut self , amount:u64,is_x:bool) -> Result<()> {

        let (from , to )  = match is_x {
            true => {
                (self.user_x.to_account_info(),
                self.vault_x.to_account_info())
            }
            false => {
                (self.user_y.to_account_info(), 
                self.vault_y.to_account_info())
            }
        };

        let cpiContext = CpiContext::new(self.token_program.to_account_info(), Transfer{
            from,
            to,
            authority:self.user.to_account_info(),
        });

        transfer(cpiContext , amount)?;

        Ok(())
    }

    pub fn withdraw_token(&mut self , amount:u64 , is_x:bool) -> Result<()> {

        let (from,to) = match is_x {
            true => {
                (self.vault_y.to_account_info() , self.user_y.to_account_info())
            }
            false => {
                (self.vault_x.to_account_info() , self.user_x.to_account_info())
            }
        };

        let seeds = &[
            &b"config"[..],
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpiContext = CpiContext::new_with_signer(self.token_program.to_account_info(), Transfer{
            from,
            to,
            authority:self.user.to_account_info(),
        },signer_seeds);

        transfer(cpiContext , amount)?;

        Ok(())
    }

}