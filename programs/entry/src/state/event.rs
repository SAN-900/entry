use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Event {
    pub organizer: Pubkey,
    pub event_id: u64,
    pub gate_authority: Pubkey,
    pub gatekeeper: Pubkey,
    pub payment_mint: Pubkey,
    pub merkle_tree: Pubkey,
    pub collection: Pubkey,
    pub price: u64,
    pub supply: u32,
    pub sold: u32,
    pub max_per_wallet: u8,
    pub resale_cap_bps: u16,
    pub royalty_bps: u16,
    #[max_len(24)]
    pub ticket_name: String,
    #[max_len(10)]
    pub ticket_symbol: String,
    #[max_len(194)]
    pub ticket_uri: String,
    pub active: bool,
    pub bump: u8,
}

impl Event {
    pub fn asset_id(&self, nonce: u64) -> Pubkey {
        mpl_bubblegum::utils::get_asset_id(&self.merkle_tree, nonce)
    }
}
