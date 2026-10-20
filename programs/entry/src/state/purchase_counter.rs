use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PurchaseCounter {
    pub event: Pubkey,
    pub wallet: Pubkey,
    pub count: u8,
    pub bump: u8,
}
