use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{burn, transfer, Burn, Mint, Token, TokenAccount, Transfer}};
use constant_product_curve::ConstantProduct;


use crate::{withdraw, Config};
use crate::error::AmmError;


#[derive(Accounts)]
pub struct Withdraw<'info>{
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

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
    )]
    pub user_lp:Account<'info , TokenAccount>,


    pub system_program: Program<'info , System>,
    pub token_program: Program<'info , Token>,
    pub associated_token_program: Program<'info , AssociatedToken>

}

impl<'info> Withdraw<'info>{
    //while working with amm you should always consider lp amount not token amount it will be hadled at client side
    //amount:lp token to burn
    //min_x:minimum amount of x user wanted
    //min_y:minimum amount of y user wanted
    pub fn withdraw(&mut self , amount:u64 , min_x:u64 , min_y:u64 ) -> Result<()> {
        require!(amount != 0 , AmmError::InvalidAmount);
        require!(self.config.locked == false , AmmError::PoolLocked);

        let amounts = ConstantProduct::xy_withdraw_amounts_from_l(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply,
            amount,
            6
        ).map_err(AmmError::from)?;

        require!(amounts.x >= min_x , AmmError::InvalidAmount);
        require!(amounts.y >= min_y , AmmError::InvalidAmount);

        self.withdraw_token(amounts.x, true);
        self.withdraw_token(amounts.y, false);
        self.burn_token(amount);
        Ok(())
    }

    pub fn withdraw_token(&mut self , amount:u64, is_x:bool) -> Result<()> {
        let (from , to )  = match is_x {
            true => {
                (self.vault_x.to_account_info(),
                self.vault_y.to_account_info())
            }
            false => {
                (self.vault_y.to_account_info(), 
                self.user_x.to_account_info())
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

    pub fn burn_token(&mut self , amount:u64) -> Result<()> {

        let cpiContext = CpiContext::new(self.token_program.to_account_info(), Burn{
            mint:self.mint_lp.to_account_info(),
            from:self.user_lp.to_account_info(),
            authority:self.user.to_account_info()
        });

        burn(cpiContext, amount)?;
        
        Ok(())
    }
}