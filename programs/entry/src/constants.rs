use anchor_lang::prelude::*;

pub const CONFIG_SEED: &[u8] = b"config";
pub const EVENT_SEED: &[u8] = b"event";
pub const LISTING_SEED: &[u8] = b"listing";
pub const CHECKIN_SEED: &[u8] = b"checkins";
pub const MAX_PLATFORM_FEE_BPS: u16 = 500;
pub const MAX_ROYALTY_BPS: u16 = 2_000;
pub const MAX_CHECK_IN_BATCH: usize = 64;
pub const BPS_DENOMINATOR: u128 = 10_000;
pub const DEFAULT_RESALE_CAP_BPS: u16 = 11_000;
pub const ACCOUNT_COMPRESSION_PROGRAM_ID: Pubkey =
    pubkey!("cmtDvXumGCrqC1Age74AVPhSRVXJMd8PJS91L8KbNCK");
pub const NOOP_PROGRAM_ID: Pubkey = pubkey!("noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV");
pub const MPL_CORE_PROGRAM_ID: Pubkey = pubkey!("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");
pub const USDC_MAINNET_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
pub const USDC_DEVNET_MINT: Pubkey = pubkey!("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");

pub fn is_supported_usdc_mint(mint: &Pubkey) -> bool {
    *mint == USDC_MAINNET_MINT || *mint == USDC_DEVNET_MINT
}
