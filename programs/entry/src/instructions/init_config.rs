use crate::{constants::*, errors::EntryError, state::Config};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    pub program: Program<'info, crate::program::Entry>,
    pub program_data: Account<'info, ProgramData>,
    /// CHECK: Fee vault receives platform fees; it need not sign.
    pub fee_vault: UncheckedAccount<'info>,
    #[account(init, payer = admin, space = 8 + Config::INIT_SPACE, seeds = [CONFIG_SEED], bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitConfig>, fee_bps: u16) -> Result<()> {
    require!(
        ctx.accounts.program.programdata_address()? == Some(ctx.accounts.program_data.key()),
        EntryError::InvalidProgramData
    );
    validate_upgrade_authority(
        ctx.accounts.admin.key(),
        ctx.accounts.program_data.upgrade_authority_address,
    )?;
    require!(fee_bps <= MAX_PLATFORM_FEE_BPS, EntryError::FeeTooHigh);
    let config = &mut ctx.accounts.config;
    config.admin = ctx.accounts.admin.key();
    config.fee_vault = ctx.accounts.fee_vault.key();
    config.fee_bps = fee_bps;
    config.bump = ctx.bumps.config;
    Ok(())
}

pub fn validate_upgrade_authority(admin: Pubkey, upgrade_authority: Option<Pubkey>) -> Result<()> {
    require!(
        upgrade_authority == Some(admin),
        EntryError::UnauthorizedConfigInitializer
    );
    Ok(())
}
