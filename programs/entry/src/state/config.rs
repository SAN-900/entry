use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub fee_vault: Pubkey,
    pub fee_bps: u16,
    pub bump: u8,
}
