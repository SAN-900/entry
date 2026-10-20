use anchor_lang::prelude::*;

use crate::{
    constants::*,
    errors::EntryError,
    state::{CheckIns, Event},
};

const MAX_SINGLE_BITMAP_SUPPLY: u32 = 80_000;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateEventArgs {
    pub event_id: u64,
    pub merkle_tree: Pubkey,
    pub registry_tree: Pubkey,
    pub registry_depth: u8,
    pub collection: Pubkey,
    pub payment_mint: Pubkey,
    pub gate_authority: Pubkey,
    pub gatekeeper: Pubkey,
    pub price: u64,
    pub supply: u32,
    pub max_per_wallet: u8,
    pub resale_cap_bps: u16,
    pub royalty_bps: u16,
    pub ticket_name: String,
    pub ticket_symbol: String,
    pub ticket_uri: String,
}

#[derive(Accounts)]
#[instruction(args: CreateEventArgs)]
pub struct CreateEvent<'info> {
    #[account(mut)]
    pub organizer: Signer<'info>,
    #[account(init, payer = organizer, space = 8 + Event::INIT_SPACE, seeds = [EVENT_SEED, organizer.key().as_ref(), args.event_id.to_le_bytes().as_ref()], bump)]
    pub event: Account<'info, Event>,
    #[account(
        init,
        payer = organizer,
        space = CheckIns::account_space(args.supply),
        seeds = [CHECKIN_SEED, event.key().as_ref()],
        bump
    )]
    pub checkins: AccountLoader<'info, CheckIns>,
    /// CHECK: Must be the event's private, empty Bubblegum V2 tree.
    #[account(address = args.merkle_tree)]
    pub merkle_tree: UncheckedAccount<'info>,
    /// CHECK: SPL Account Compression tree storing current owner-of-record leaves.
    #[account(mut, address = args.registry_tree)]
    pub registry_tree: UncheckedAccount<'info>,
    /// CHECK: Canonical Bubblegum V2 tree config PDA.
    #[account(seeds = [args.merkle_tree.as_ref()], bump, seeds::program = bubblegum_program.key())]
    pub tree_config: UncheckedAccount<'info>,
    /// CHECK: Must be an MPL Core collection with BubblegumV2 and event-controlled permanent delegates.
    #[account(address = args.collection)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: Fixed Bubblegum V2 program ID.
    #[account(address = mpl_bubblegum::ID)]
    pub bubblegum_program: UncheckedAccount<'info>,
    /// CHECK: Fixed MPL Core program ID.
    #[account(address = MPL_CORE_PROGRAM_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    /// CHECK: Fixed SPL Account Compression program.
    #[account(address = ACCOUNT_COMPRESSION_PROGRAM_ID)]
    pub compression_program: UncheckedAccount<'info>,
    /// CHECK: Fixed SPL noop/log-wrapper program.
    #[account(address = NOOP_PROGRAM_ID)]
    pub log_wrapper: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateEvent>, args: CreateEventArgs) -> Result<()> {
    require!(args.price > 0, EntryError::InvalidPrice);
    require!(
        args.supply > 0 && args.supply <= MAX_SINGLE_BITMAP_SUPPLY,
        EntryError::InvalidSupply
    );
    // SPL ConcurrentMerkleTree 1.0 supports depth 14 and 20 with the
    // protocol's fixed 64-entry changelog buffer. Pick the smallest supported
    // tree that can contain this event's declared inventory.
    require!(
        (args.supply <= (1 << 14) && args.registry_depth == 14)
            || (args.supply > (1 << 14) && args.registry_depth == 20),
        EntryError::InvalidRegistryTree
    );
    require!(args.max_per_wallet > 0, EntryError::InvalidSupply);
    require!(
        is_supported_usdc_mint(&args.payment_mint),
        EntryError::InvalidPaymentMint
    );
    require!(args.resale_cap_bps >= 10_000, EntryError::InvalidResaleCap);
    require!(
        args.royalty_bps <= MAX_ROYALTY_BPS,
        EntryError::InvalidRoyalty
    );
    require!(
        u32::from(args.royalty_bps) + u32::from(MAX_PLATFORM_FEE_BPS) <= 10_000,
        EntryError::InvalidRoyalty
    );
    require!(
        !args.gate_authority.eq(&Pubkey::default())
            && !args.gatekeeper.eq(&Pubkey::default())
            && !args.ticket_name.is_empty()
            && !args.ticket_symbol.is_empty()
            && !args.ticket_uri.is_empty()
            && args.ticket_name.len() <= 24
            && args.ticket_symbol.len() <= 10
            // Leave room for the slash and a five-digit nonce appended during minting.
            && args.ticket_uri.len() <= 194,
        EntryError::InvalidMetadata
    );
    require_keys_eq!(
        *ctx.accounts.merkle_tree.owner,
        ACCOUNT_COMPRESSION_PROGRAM_ID,
        EntryError::InvalidEventTree
    );
    require_keys_neq!(
        ctx.accounts.registry_tree.key(),
        ctx.accounts.merkle_tree.key(),
        EntryError::InvalidRegistryTree
    );
    require_keys_eq!(
        *ctx.accounts.registry_tree.owner,
        ACCOUNT_COMPRESSION_PROGRAM_ID,
        EntryError::InvalidRegistryTree
    );
    require_keys_eq!(
        *ctx.accounts.collection.owner,
        MPL_CORE_PROGRAM_ID,
        EntryError::InvalidAssetId
    );
    crate::utils::core_collection::validate_collection(
        &ctx.accounts
            .collection
            .try_borrow_data()
            .map_err(|_| error!(EntryError::InvalidAssetId))?,
        ctx.accounts.event.key(),
    )?;
    require_keys_eq!(
        *ctx.accounts.tree_config.owner,
        mpl_bubblegum::ID,
        EntryError::InvalidEventTree
    );
    let tree_config =
        mpl_bubblegum::accounts::TreeConfig::try_from(&ctx.accounts.tree_config.to_account_info())
            .map_err(|_| error!(EntryError::InvalidEventTree))?;
    crate::utils::registry::validate_bubblegum_tree(
        &ctx.accounts
            .merkle_tree
            .try_borrow_data()
            .map_err(|_| error!(EntryError::InvalidEventTree))?,
        ctx.accounts.tree_config.key(),
    )?;
    require!(
        tree_config.tree_delegate == ctx.accounts.event.key()
            && !tree_config.is_public
            && tree_config.version == mpl_bubblegum::types::Version::V2
            && tree_config.num_minted == 0
            && u64::from(args.supply) <= tree_config.total_mint_capacity,
        EntryError::InvalidEventTree
    );

    {
        let event = &mut ctx.accounts.event;
        event.organizer = ctx.accounts.organizer.key();
        event.event_id = args.event_id;
        event.gate_authority = args.gate_authority;
        event.gatekeeper = args.gatekeeper;
        event.payment_mint = args.payment_mint;
        event.merkle_tree = args.merkle_tree;
        event.registry_tree = args.registry_tree;
        event.registry_depth = args.registry_depth;
        event.registry_seq = 0;
        event.collection = args.collection;
        event.price = args.price;
        event.supply = args.supply;
        event.sold = 0;
        event.max_per_wallet = args.max_per_wallet;
        event.resale_cap_bps = args.resale_cap_bps;
        event.royalty_bps = args.royalty_bps;
        event.ticket_name = args.ticket_name;
        event.ticket_symbol = args.ticket_symbol;
        event.ticket_uri = args.ticket_uri;
        event.active = true;
        event.bump = ctx.bumps.event;
    }
    let event_bump = [ctx.accounts.event.bump];
    let event_seeds: &[&[u8]] = &[
        EVENT_SEED,
        ctx.accounts.event.organizer.as_ref(),
        &ctx.accounts.event.event_id.to_le_bytes(),
        &event_bump,
    ];
    crate::utils::registry::initialize_tree(
        &ctx.accounts.compression_program.to_account_info(),
        &ctx.accounts.registry_tree.to_account_info(),
        &ctx.accounts.event.to_account_info(),
        &ctx.accounts.log_wrapper.to_account_info(),
        ctx.accounts.event.registry_depth,
        event_seeds,
    )?;
    ctx.accounts.event.registry_root =
        crate::utils::registry::empty_root(ctx.accounts.event.registry_depth);
    crate::state::checkin::initialize(
        &ctx.accounts.checkins,
        ctx.accounts.event.key(),
        ctx.accounts.event.supply,
    )?;
    Ok(())
}
