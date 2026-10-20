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

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CheckInTicket {
    pub nonce: u64,
    pub owner: Pubkey,
}

pub fn handler(ctx: Context<CheckInBatch>, tickets: Vec<CheckInTicket>) -> Result<()> {
    require!(!tickets.is_empty(), EntryError::InvalidSupply);
    require!(
        tickets.len() <= MAX_CHECK_IN_BATCH,
        EntryError::CheckInBatchTooLarge
    );
    let depth = usize::from(ctx.accounts.event.registry_depth);
    require!(depth == 14 || depth == 20, EntryError::InvalidRegistryProof);
    let max_tickets_for_tree = (60 / depth).max(1);
    require!(
        tickets.len() <= max_tickets_for_tree
            && ctx.remaining_accounts.len() == tickets.len() * depth
            && ctx
                .remaining_accounts
                .iter()
                .all(|account| !account.is_signer && !account.is_writable),
        EntryError::InvalidRegistryProof
    );
    crate::state::checkin::validate(
        &ctx.accounts.checkins,
        ctx.accounts.event.key(),
        ctx.accounts.event.supply,
    )?;
    let now = Clock::get()?.unix_timestamp;

    for (position, ticket) in tickets.into_iter().enumerate() {
        let nonce = ticket.nonce;
        require!(
            nonce < u64::from(ctx.accounts.event.sold),
            EntryError::TicketIdOutOfRange
        );
        let proof_start = position * depth;
        let proof = &ctx.remaining_accounts[proof_start..proof_start + depth];
        let owner_leaf =
            crate::utils::registry::owner_leaf(ctx.accounts.event.key(), nonce, ticket.owner);
        crate::utils::registry::root_from_proof(
            ctx.accounts.event.registry_root,
            nonce,
            proof,
            owner_leaf,
        )?;
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
