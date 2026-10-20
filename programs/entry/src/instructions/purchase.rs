use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};
use mpl_bubblegum::{
    instructions::{FreezeV2CpiBuilder, MintV2CpiBuilder},
    types::{Creator, MetadataArgsV2, TokenStandard},
};

use crate::{
    constants::*,
    errors::EntryError,
    state::{Config, Event, PurchaseCounter},
};

const MPL_CORE_CPI_SIGNER: Pubkey = pubkey!("CbNY3JiXdXNE9tPNEk1aRZVEkWdj2v7kfJLNQwZZgpXk");

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PurchaseArgs {
    /// Protects the buyer's precomputed append proof from tree races.
    pub expected_nonce: u64,
    /// Root expected immediately after minting this leaf; Bubblegum verifies it during freeze.
    pub freeze_root: [u8; 32],
}

#[derive(Accounts)]
#[instruction(args: PurchaseArgs)]
pub struct Purchase<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    pub gatekeeper: Signer<'info>,
    #[account(
        mut,
        seeds = [EVENT_SEED, event.organizer.as_ref(), event.event_id.to_le_bytes().as_ref()],
        bump = event.bump,
        constraint = event.active @ EntryError::EventInactive,
        constraint = event.gatekeeper == gatekeeper.key() @ EntryError::UnauthorizedGatekeeper
    )]
    pub event: Box<Account<'info, Event>>,
    #[account(
        init_if_needed,
        payer = buyer,
        space = 8 + PurchaseCounter::INIT_SPACE,
        seeds = [b"purchases", event.key().as_ref(), buyer.key().as_ref()],
        bump
    )]
    pub purchase_counter: Box<Account<'info, PurchaseCounter>>,
    #[account(seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(address = event.payment_mint, constraint = is_supported_usdc_mint(&payment_mint.key()) @ EntryError::InvalidPaymentMint, constraint = payment_mint.decimals == 6 @ EntryError::InvalidPaymentMint)]
    pub payment_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub buyer_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub organizer_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub fee_vault_usdc: Box<Account<'info, TokenAccount>>,
    /// CHECK: Bubblegum V2 tree config PDA; parsed and checked against the event before mint.
    #[account(mut, seeds = [event.merkle_tree.as_ref()], bump, seeds::program = bubblegum_program.key())]
    pub tree_config: UncheckedAccount<'info>,
    /// CHECK: The event's private Bubblegum V2 Merkle tree.
    #[account(mut, address = event.merkle_tree)]
    pub merkle_tree: UncheckedAccount<'info>,
    /// CHECK: The event's MPL Core collection.
    #[account(mut, address = event.collection)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: Canonical Bubblegum V2 CPI signer PDA for MPL Core.
    #[account(address = MPL_CORE_CPI_SIGNER)]
    pub mpl_core_cpi_signer: UncheckedAccount<'info>,
    /// CHECK: Fixed Bubblegum V2 program.
    #[account(address = mpl_bubblegum::ID)]
    pub bubblegum_program: UncheckedAccount<'info>,
    /// CHECK: Fixed SPL account-compression program.
    #[account(address = ACCOUNT_COMPRESSION_PROGRAM_ID)]
    pub compression_program: UncheckedAccount<'info>,
    /// CHECK: Fixed SPL noop/log-wrapper program.
    #[account(address = NOOP_PROGRAM_ID)]
    pub log_wrapper: UncheckedAccount<'info>,
    /// CHECK: Fixed MPL Core program.
    #[account(address = MPL_CORE_PROGRAM_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, Purchase<'info>>,
    args: PurchaseArgs,
) -> Result<()> {
    require!(
        ctx.accounts.event.sold < ctx.accounts.event.supply,
        EntryError::SoldOut
    );
    crate::instructions::buy_resale::validate_usdc_ata(
        &ctx.accounts.buyer_usdc,
        ctx.accounts.payment_mint.key(),
        ctx.accounts.buyer.key(),
    )?;
    crate::instructions::buy_resale::validate_usdc_ata(
        &ctx.accounts.organizer_usdc,
        ctx.accounts.payment_mint.key(),
        ctx.accounts.event.organizer,
    )?;
    crate::instructions::buy_resale::validate_usdc_ata(
        &ctx.accounts.fee_vault_usdc,
        ctx.accounts.payment_mint.key(),
        ctx.accounts.config.fee_vault,
    )?;
    let tree_config =
        mpl_bubblegum::accounts::TreeConfig::try_from(&ctx.accounts.tree_config.to_account_info())
            .map_err(|_| error!(EntryError::InvalidEventTree))?;
    require!(
        *ctx.accounts.tree_config.owner == mpl_bubblegum::ID
            && tree_config.tree_delegate == ctx.accounts.event.key()
            && !tree_config.is_public
            && tree_config.num_minted == u64::from(ctx.accounts.event.sold),
        EntryError::InvalidEventTree
    );
    let nonce = tree_config.num_minted;
    require!(
        args.expected_nonce == nonce,
        EntryError::InvalidBubblegumProof
    );
    let index = u32::try_from(nonce).map_err(|_| error!(EntryError::ArithmeticOverflow))?;

    let counter = &mut ctx.accounts.purchase_counter;
    if counter.event == Pubkey::default() {
        counter.event = ctx.accounts.event.key();
        counter.wallet = ctx.accounts.buyer.key();
        counter.count = 0;
        counter.bump = ctx.bumps.purchase_counter;
    }
    require!(
        counter.event == ctx.accounts.event.key(),
        EntryError::InvalidCheckInAccount
    );
    require!(
        counter.wallet == ctx.accounts.buyer.key(),
        EntryError::InvalidCheckInAccount
    );
    require!(
        counter.count < ctx.accounts.event.max_per_wallet,
        EntryError::InvalidSupply
    );

    let amount = ctx.accounts.event.price;
    let fee = crate::utils::pricing::percentage_amount(amount, ctx.accounts.config.fee_bps)?;
    let organizer_amount = amount
        .checked_sub(fee)
        .ok_or(EntryError::ArithmeticOverflow)?;
    transfer_checked(
        &ctx.accounts.token_program,
        &ctx.accounts.buyer_usdc,
        &ctx.accounts.payment_mint,
        &ctx.accounts.buyer,
        &ctx.accounts.organizer_usdc,
        organizer_amount,
    )?;
    transfer_checked(
        &ctx.accounts.token_program,
        &ctx.accounts.buyer_usdc,
        &ctx.accounts.payment_mint,
        &ctx.accounts.buyer,
        &ctx.accounts.fee_vault_usdc,
        fee,
    )?;

    let metadata = MetadataArgsV2 {
        name: format!("{} #{}", ctx.accounts.event.ticket_name, nonce + 1),
        symbol: ctx.accounts.event.ticket_symbol.clone(),
        uri: format!(
            "{}/{}",
            ctx.accounts.event.ticket_uri.trim_end_matches('/'),
            nonce
        ),
        seller_fee_basis_points: ctx.accounts.event.royalty_bps,
        primary_sale_happened: false,
        is_mutable: false,
        token_standard: Some(TokenStandard::NonFungible),
        creators: vec![Creator {
            address: ctx.accounts.event.key(),
            verified: true,
            share: 100,
        }],
        collection: Some(ctx.accounts.event.collection),
    };
    require!(
        metadata.name.len() <= 32 && metadata.uri.len() <= 200,
        EntryError::InvalidMetadata
    );
    let data_hash = mpl_bubblegum::hash::hash_metadata(&metadata)
        .map_err(|_| error!(EntryError::InvalidBubblegumProof))?;
    let creator_hash = mpl_bubblegum::hash::hash_creators(&metadata.creators);
    let asset_data_hash = mpl_bubblegum::hash::hash_asset_data_option(None)
        .map_err(|_| error!(EntryError::InvalidBubblegumProof))?;

    let bump = [ctx.accounts.event.bump];
    let event_seeds: &[&[u8]] = &[
        EVENT_SEED,
        ctx.accounts.event.organizer.as_ref(),
        &ctx.accounts.event.event_id.to_le_bytes(),
        &bump,
    ];
    let bubblegum_program = ctx.accounts.bubblegum_program.to_account_info();
    let tree_config_info = ctx.accounts.tree_config.to_account_info();
    let buyer = ctx.accounts.buyer.to_account_info();
    let event_authority = ctx.accounts.event.to_account_info();
    let merkle_tree = ctx.accounts.merkle_tree.to_account_info();
    let collection = ctx.accounts.collection.to_account_info();
    let core_cpi_signer = ctx.accounts.mpl_core_cpi_signer.to_account_info();
    let log_wrapper = ctx.accounts.log_wrapper.to_account_info();
    let compression_program = ctx.accounts.compression_program.to_account_info();
    let mpl_core_program = ctx.accounts.mpl_core_program.to_account_info();
    let system_program = ctx.accounts.system_program.to_account_info();

    let mut mint = MintV2CpiBuilder::new(&bubblegum_program);
    mint.tree_config(&tree_config_info)
        .payer(&buyer)
        .tree_creator_or_delegate(Some(&event_authority))
        .collection_authority(Some(&event_authority))
        .leaf_owner(&buyer)
        .leaf_delegate(None)
        .merkle_tree(&merkle_tree)
        .core_collection(Some(&collection))
        .mpl_core_cpi_signer(Some(&core_cpi_signer))
        .log_wrapper(&log_wrapper)
        .compression_program(&compression_program)
        .mpl_core_program(&mpl_core_program)
        .system_program(&system_program)
        .metadata(metadata.clone());
    mint.invoke_signed(&[event_seeds])?;

    require!(
        ctx.remaining_accounts.len() <= 32
            && ctx
                .remaining_accounts
                .iter()
                .all(|account| !account.is_signer && !account.is_writable),
        EntryError::InvalidMerkleProofAccounts
    );
    let proof_accounts: Vec<_> = ctx
        .remaining_accounts
        .iter()
        .map(|account| (account, false, false))
        .collect();
    let mut freeze = FreezeV2CpiBuilder::new(&bubblegum_program);
    freeze
        .tree_config(&tree_config_info)
        .payer(&buyer)
        .authority(Some(&event_authority))
        .leaf_owner(&buyer)
        .leaf_delegate(&buyer)
        .merkle_tree(&merkle_tree)
        .core_collection(Some(&collection))
        .log_wrapper(&log_wrapper)
        .compression_program(&compression_program)
        .system_program(&system_program)
        .root(args.freeze_root)
        .data_hash(data_hash)
        .creator_hash(creator_hash)
        .asset_data_hash(asset_data_hash)
        .flags(0)
        .nonce(nonce)
        .index(index)
        .add_remaining_accounts(&proof_accounts)
        .invoke_signed(&[event_seeds])?;

    counter.count = counter
        .count
        .checked_add(1)
        .ok_or(EntryError::ArithmeticOverflow)?;
    ctx.accounts.event.sold = ctx
        .accounts
        .event
        .sold
        .checked_add(1)
        .ok_or(EntryError::ArithmeticOverflow)?;
    emit!(TicketPurchased {
        event: ctx.accounts.event.key(),
        buyer: ctx.accounts.buyer.key(),
        nonce,
        asset_id: mpl_bubblegum::utils::get_asset_id(&ctx.accounts.event.merkle_tree, nonce),
        price: amount,
        fee,
    });
    Ok(())
}

#[event]
pub struct TicketPurchased {
    pub event: Pubkey,
    pub buyer: Pubkey,
    pub nonce: u64,
    pub asset_id: Pubkey,
    pub price: u64,
    pub fee: u64,
}

fn transfer_checked<'info>(
    token_program: &Program<'info, Token>,
    source: &Account<'info, TokenAccount>,
    mint: &Account<'info, Mint>,
    buyer: &Signer<'info>,
    destination: &Account<'info, TokenAccount>,
    amount: u64,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    token::transfer_checked(
        CpiContext::new(
            token_program.to_account_info(),
            TransferChecked {
                from: source.to_account_info(),
                mint: mint.to_account_info(),
                to: destination.to_account_info(),
                authority: buyer.to_account_info(),
            },
        ),
        amount,
        mint.decimals,
    )
}
