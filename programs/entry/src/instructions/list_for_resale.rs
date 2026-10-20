use anchor_lang::prelude::*;

use crate::{
    constants::*,
    errors::EntryError,
    state::{CheckIns, Event, Listing},
    utils::pricing::resale_cap,
};

#[derive(Accounts)]
#[instruction(nonce: u64, ask_price: u64)]
pub struct ListForResale<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(
        seeds = [EVENT_SEED, event.organizer.as_ref(), event.event_id.to_le_bytes().as_ref()],
        bump = event.bump,
        constraint = event.active @ EntryError::EventInactive
    )]
    pub event: Account<'info, Event>,
    #[account(seeds = [CHECKIN_SEED, event.key().as_ref()], bump)]
    pub checkins: AccountLoader<'info, CheckIns>,
    #[account(
        init,
        payer = seller,
        space = 8 + Listing::INIT_SPACE,
        seeds = [
            LISTING_SEED,
            event.key().as_ref(),
            event.asset_id(nonce).as_ref(),
            seller.key().as_ref()
        ],
        bump
    )]
    pub listing: Account<'info, Listing>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ListForResale>, nonce: u64, ask_price: u64) -> Result<()> {
    require!(ask_price > 0, EntryError::InvalidPrice);
    require!(
        nonce < u64::from(ctx.accounts.event.sold),
        EntryError::TicketIdOutOfRange
    );
    crate::state::checkin::validate(
        &ctx.accounts.checkins,
        ctx.accounts.event.key(),
        ctx.accounts.event.supply,
    )?;
    require!(
        !crate::state::checkin::is_checked_in(&ctx.accounts.checkins, nonce)?,
        EntryError::TicketAlreadyCheckedIn
    );
    require!(
        ask_price <= resale_cap(ctx.accounts.event.price, ctx.accounts.event.resale_cap_bps)?,
        EntryError::ResaleCapExceeded
    );

    let listing = &mut ctx.accounts.listing;
    listing.event = ctx.accounts.event.key();
    listing.asset_id = ctx.accounts.event.asset_id(nonce);
    listing.leaf_nonce = nonce;
    listing.seller = ctx.accounts.seller.key();
    listing.ask_price = ask_price;
    listing.bump = ctx.bumps.listing;
    Ok(())
}
