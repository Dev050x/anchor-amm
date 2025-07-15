use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config{
    pub seed:u64,
    pub mint_x:Pubkey,
    pub mint_y:Pubkey,
    pub authority:Option<Pubkey>,
    pub fee: u16,
    pub config_bump:u8,
    pub lp_bum:u8,
    pub locked:bool
}