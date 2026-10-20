use anchor_lang::prelude::*;

use crate::{
    constants::*,
    errors::EntryError,
    state::{Event, Listing},
};

#[derive(Accounts)]
pub struct CancelListing<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(seeds = [EVENT_SEED, event.organizer.as_ref(), event.event_id.to_le_bytes().as_ref()], bump = event.bump)]
    pub event: Account<'info, Event>,
    #[account(
        mut,
        close = seller,
        seeds = [LISTING_SEED, event.key().as_ref(), listing.asset_id.as_ref(), seller.key().as_ref()],
        bump = listing.bump,
        has_one = event @ EntryError::InvalidAssetId,
        has_one = seller @ EntryError::NotListingSeller
    )]
    pub listing: Account<'info, Listing>,
}

pub fn handler(_ctx: Context<CancelListing>) -> Result<()> {
    Ok(())
}
