use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, TransferChecked},
};

use crate::{
    constants::*,
    errors::EntryError,
    state::{CheckIns, Config, Event, Listing, PurchaseCounter},
    utils::{
        bubblegum::{transfer_frozen_ticket, TicketTransferProofs},
        pricing::{resale_cap, resale_split},
    },
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BuyResaleArgs {
    pub proofs: TicketTransferProofs,
}

#[derive(Accounts)]
#[instruction(args: BuyResaleArgs)]
pub struct BuyResale<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(seeds = [EVENT_SEED, event.organizer.as_ref(), event.event_id.to_le_bytes().as_ref()], bump = event.bump, constraint = event.active @ EntryError::EventInactive)]
    pub event: Account<'info, Event>,
    #[account(
        mut,
        close = seller,
        seeds = [LISTING_SEED, event.key().as_ref(), listing.asset_id.as_ref(), listing.seller.as_ref()],
        bump = listing.bump,
        has_one = event @ EntryError::InvalidAssetId
    )]
    pub listing: Account<'info, Listing>,
    /// CHECK: Listing PDA binds this key as seller; receives token proceeds and listing rent.
    #[account(mut, address = listing.seller)]
    pub seller: UncheckedAccount<'info>,
    #[account(seeds = [CHECKIN_SEED, event.key().as_ref()], bump)]
    pub checkins: AccountLoader<'info, CheckIns>,
    #[account(
        init_if_needed,
        payer = buyer,
        space = 8 + PurchaseCounter::INIT_SPACE,
        seeds = [b"purchases", event.key().as_ref(), buyer.key().as_ref()],
        bump
    )]
    pub purchase_counter: Account<'info, PurchaseCounter>,
    #[account(seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(address = event.payment_mint, constraint = is_supported_usdc_mint(&payment_mint.key()) @ EntryError::InvalidPaymentMint, constraint = payment_mint.decimals == 6 @ EntryError::InvalidPaymentMint)]
    pub payment_mint: Account<'info, Mint>,
    #[account(mut, associated_token::mint = payment_mint, associated_token::authority = buyer, associated_token::token_program = token_program)]
    pub buyer_usdc: Account<'info, TokenAccount>,
    #[account(mut, associated_token::mint = payment_mint, associated_token::authority = seller, associated_token::token_program = token_program)]
    pub seller_usdc: Account<'info, TokenAccount>,
    #[account(mut, associated_token::mint = payment_mint, associated_token::authority = event.organizer, associated_token::token_program = token_program)]
    pub organizer_usdc: Account<'info, TokenAccount>,
    #[account(mut, associated_token::mint = payment_mint, associated_token::authority = config.fee_vault, associated_token::token_program = token_program)]
    pub fee_vault_usdc: Account<'info, TokenAccount>,
    /// CHECK: Bubblegum V2 tree config PDA.
    #[account(seeds = [event.merkle_tree.as_ref()], bump, seeds::program = bubblegum_program.key())]
    pub tree_config: UncheckedAccount<'info>,
    /// CHECK: Configured event Merkle tree.
    #[account(mut, address = event.merkle_tree)]
    pub merkle_tree: UncheckedAccount<'info>,
    /// CHECK: Configured MPL Core collection.
    #[account(address = event.collection)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: Must be the current owner represented in the Bubblegum leaf proof.
    #[account(address = listing.seller)]
    pub leaf_owner: UncheckedAccount<'info>,
    /// CHECK: Event PDA is tree delegate and both permanent collection delegates.
    #[account(address = event.key())]
    pub event_authority: UncheckedAccount<'info>,
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
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, BuyResale<'info>>,
    args: BuyResaleArgs,
) -> Result<()> {
    require!(
        ctx.accounts.buyer.key() != ctx.accounts.listing.seller,
        EntryError::SelfPurchase
    );
    require!(
        ctx.accounts.listing.leaf_nonce < u64::from(ctx.accounts.event.sold),
        EntryError::TicketIdOutOfRange
    );
    crate::state::checkin::validate(
        &ctx.accounts.checkins,
        ctx.accounts.event.key(),
        ctx.accounts.event.supply,
    )?;
    require!(
        !crate::state::checkin::is_checked_in(
            &ctx.accounts.checkins,
            ctx.accounts.listing.leaf_nonce
        )?,
        EntryError::TicketAlreadyCheckedIn
    );
    require!(
        args.proofs.thaw.nonce == ctx.accounts.listing.leaf_nonce,
        EntryError::InvalidBubblegumProof
    );
    require!(
        args.proofs.thaw.index
            == u32::try_from(ctx.accounts.listing.leaf_nonce)
                .map_err(|_| error!(EntryError::ArithmeticOverflow))?,
        EntryError::InvalidBubblegumProof
    );
    require!(
        mpl_bubblegum::utils::get_asset_id(
            &ctx.accounts.event.merkle_tree,
            ctx.accounts.listing.leaf_nonce
        ) == ctx.accounts.listing.asset_id,
        EntryError::InvalidAssetId
    );
    require!(
        ctx.accounts.listing.ask_price
            <= resale_cap(ctx.accounts.event.price, ctx.accounts.event.resale_cap_bps)?,
        EntryError::ResaleCapExceeded
    );

    let tree_config =
        mpl_bubblegum::accounts::TreeConfig::try_from(&ctx.accounts.tree_config.to_account_info())
            .map_err(|_| error!(EntryError::InvalidEventTree))?;
    require!(
        *ctx.accounts.tree_config.owner == mpl_bubblegum::ID
            && tree_config.tree_delegate == ctx.accounts.event.key()
            && !tree_config.is_public,
        EntryError::InvalidEventTree
    );

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

    let ask = ctx.accounts.listing.ask_price;
    let (seller_amount, royalty, fee) = resale_split(
        ask,
        ctx.accounts.event.royalty_bps,
        ctx.accounts.config.fee_bps,
    )?;
    transfer_checked(
        &ctx.accounts.token_program,
        &ctx.accounts.buyer_usdc,
        &ctx.accounts.payment_mint,
        &ctx.accounts.buyer,
        &ctx.accounts.seller_usdc,
        seller_amount,
    )?;
    transfer_checked(
        &ctx.accounts.token_program,
        &ctx.accounts.buyer_usdc,
        &ctx.accounts.payment_mint,
        &ctx.accounts.buyer,
        &ctx.accounts.organizer_usdc,
        royalty,
    )?;
    transfer_checked(
        &ctx.accounts.token_program,
        &ctx.accounts.buyer_usdc,
        &ctx.accounts.payment_mint,
        &ctx.accounts.buyer,
        &ctx.accounts.fee_vault_usdc,
        fee,
    )?;

    let bump = [ctx.accounts.event.bump];
    let event_seeds: &[&[u8]] = &[
        EVENT_SEED,
        ctx.accounts.event.organizer.as_ref(),
        &ctx.accounts.event.event_id.to_le_bytes(),
        &bump,
    ];
    transfer_frozen_ticket(
        &ctx.accounts.bubblegum_program.to_account_info(),
        &ctx.accounts.tree_config.to_account_info(),
        &ctx.accounts.buyer.to_account_info(),
        &ctx.accounts.leaf_owner.to_account_info(),
        &ctx.accounts.buyer.to_account_info(),
        &ctx.accounts.event_authority.to_account_info(),
        &ctx.accounts.merkle_tree.to_account_info(),
        &ctx.accounts.collection.to_account_info(),
        &ctx.accounts.log_wrapper.to_account_info(),
        &ctx.accounts.compression_program.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        &args.proofs,
        ctx.remaining_accounts,
        event_seeds,
    )?;

    counter.count = counter
        .count
        .checked_add(1)
        .ok_or(EntryError::ArithmeticOverflow)?;
    Ok(())
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
