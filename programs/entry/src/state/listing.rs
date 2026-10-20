use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Listing {
    pub event: Pubkey,
    pub asset_id: Pubkey,
    pub leaf_nonce: u64,
    pub seller: Pubkey,
    pub ask_price: u64,
    pub bump: u8,
}
