use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

declare_id!("EsW1tbFTY87F7k669NzJ1mdp7kVgTb2DXyANvpSoU2y9");

#[program]
pub mod entry {
    use super::*;

    pub fn init_config(ctx: Context<InitConfig>, fee_bps: u16) -> Result<()> {
        instructions::init_config::handler(ctx, fee_bps)
    }

    pub fn create_event(ctx: Context<CreateEvent>, args: CreateEventArgs) -> Result<()> {
        instructions::create_event::handler(ctx, args)
    }

    pub fn list_for_resale(ctx: Context<ListForResale>, nonce: u64, ask_price: u64) -> Result<()> {
        instructions::list_for_resale::handler(ctx, nonce, ask_price)
    }

    pub fn cancel_listing(ctx: Context<CancelListing>) -> Result<()> {
        instructions::cancel_listing::handler(ctx)
    }

    pub fn check_in_batch(
        ctx: Context<CheckInBatch>,
        tickets: Vec<instructions::check_in::CheckInTicket>,
    ) -> Result<()> {
        instructions::check_in::handler(ctx, tickets)
    }

    pub fn purchase<'info>(
        ctx: Context<'_, '_, '_, 'info, Purchase<'info>>,
        args: PurchaseArgs,
    ) -> Result<()> {
        instructions::purchase::handler(ctx, args)
    }

    pub fn buy_resale<'info>(
        ctx: Context<'_, '_, '_, 'info, BuyResale<'info>>,
        args: BuyResaleArgs,
    ) -> Result<()> {
        instructions::buy_resale::handler(ctx, args)
    }

    pub fn set_gate_authority(
        ctx: Context<SetGateAuthority>,
        gate_authority: Pubkey,
    ) -> Result<()> {
        instructions::set_gate_authority::handler(ctx, gate_authority)
    }

    pub fn set_active(ctx: Context<SetActive>, active: bool) -> Result<()> {
        instructions::set_active::handler(ctx, active)
    }
}
