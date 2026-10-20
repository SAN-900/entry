use anchor_lang::prelude::*;

use crate::{
    constants::*,
    errors::EntryError,
    state::{CheckIns, Event},
};

#[event]
pub struct CheckInBatchEvent {
    pub event: Pubkey,
    pub nonce: u64,
    pub gate_authority: Pubkey,
    pub checked_in_at: i64,
    pub already_checked_in: bool,
}

#[derive(Accounts)]
pub struct CheckInBatch<'info> {
    pub gate: Signer<'info>,
    #[account(
        seeds = [EVENT_SEED, event.organizer.as_ref(), event.event_id.to_le_bytes().as_ref()],
        bump = event.bump,
        constraint = event.active @ EntryError::EventInactive,
        constraint = event.gate_authority == gate.key() @ EntryError::UnauthorizedGate
    )]
    pub event: Account<'info, Event>,
    #[account(mut, seeds = [CHECKIN_SEED, event.key().as_ref()], bump)]
    pub checkins: AccountLoader<'info, CheckIns>,
}

pub fn handler(ctx: Context<CheckInBatch>, nonces: Vec<u64>) -> Result<()> {
    require!(!nonces.is_empty(), EntryError::InvalidSupply);
    require!(
        nonces.len() <= MAX_CHECK_IN_BATCH,
        EntryError::CheckInBatchTooLarge
    );
    crate::state::checkin::validate(
        &ctx.accounts.checkins,
        ctx.accounts.event.key(),
        ctx.accounts.event.supply,
    )?;
    let now = Clock::get()?.unix_timestamp;

    for nonce in nonces {
        require!(
            nonce < u64::from(ctx.accounts.event.sold),
            EntryError::TicketIdOutOfRange
        );
        let was_checked_in = crate::state::checkin::is_checked_in(&ctx.accounts.checkins, nonce)?;
        if !was_checked_in {
            crate::state::checkin::set_checked_in(&ctx.accounts.checkins, nonce)?;
        }
        emit!(CheckInBatchEvent {
            event: ctx.accounts.event.key(),
            nonce,
            gate_authority: ctx.accounts.gate.key(),
            checked_in_at: now,
            already_checked_in: was_checked_in,
        });
    }
    Ok(())
}
