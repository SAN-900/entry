use anchor_lang::prelude::*;

use crate::{constants::EVENT_SEED, errors::EntryError, state::Event};

#[derive(Accounts)]
pub struct SetActive<'info> {
    pub organizer: Signer<'info>,
    #[account(
        mut,
        seeds = [EVENT_SEED, organizer.key().as_ref(), event.event_id.to_le_bytes().as_ref()],
        bump = event.bump,
        has_one = organizer @ EntryError::UnauthorizedOrganizer
    )]
    pub event: Account<'info, Event>,
}

pub fn handler(ctx: Context<SetActive>, active: bool) -> Result<()> {
    ctx.accounts.event.active = active;
    Ok(())
}
