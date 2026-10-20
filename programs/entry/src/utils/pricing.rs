use crate::{constants::BPS_DENOMINATOR, errors::EntryError};
use anchor_lang::prelude::*;

pub fn resale_cap(face_price: u64, cap_bps: u16) -> Result<u64> {
    let product = (face_price as u128)
        .checked_mul(cap_bps as u128)
        .ok_or(EntryError::ArithmeticOverflow)?;
    let amount = product / BPS_DENOMINATOR;
    u64::try_from(amount).map_err(|_| error!(EntryError::ArithmeticOverflow))
}

pub fn royalty_amount(ask_price: u64, royalty_bps: u16) -> Result<u64> {
    let product = (ask_price as u128)
        .checked_mul(royalty_bps as u128)
        .ok_or(EntryError::ArithmeticOverflow)?;
    u64::try_from(product / BPS_DENOMINATOR).map_err(|_| error!(EntryError::ArithmeticOverflow))
}

pub fn resale_split(ask_price: u64, royalty_bps: u16, fee_bps: u16) -> Result<(u64, u64, u64)> {
    let royalty = royalty_amount(ask_price, royalty_bps)?;
    let fee = percentage_amount(ask_price, fee_bps)?;
    let seller = ask_price
        .checked_sub(royalty)
        .and_then(|amount| amount.checked_sub(fee))
        .ok_or(EntryError::ArithmeticOverflow)?;
    Ok((seller, royalty, fee))
}

pub fn percentage_amount(amount: u64, bps: u16) -> Result<u64> {
    let product = u128::from(amount)
        .checked_mul(u128::from(bps))
        .ok_or(EntryError::ArithmeticOverflow)?;
    u64::try_from(product / BPS_DENOMINATOR).map_err(|_| error!(EntryError::ArithmeticOverflow))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resale_cap_is_always_based_on_face_price() {
        assert_eq!(resale_cap(10_000_000, 11_000).unwrap(), 11_000_000);
        assert_eq!(resale_cap(11_000_000, 11_000).unwrap(), 12_100_000);
    }

    #[test]
    fn settlement_split_has_no_dust() {
        let ask = 10_001_003;
        let (seller, royalty, fee) = resale_split(ask, 750, 500).unwrap();
        assert_eq!(seller + royalty + fee, ask);
        assert_eq!(royalty, ask * 750 / 10_000);
        assert_eq!(fee, ask * 500 / 10_000);
    }

    #[test]
    fn rejects_split_when_bps_exceed_total() {
        assert!(resale_split(100, 8_000, 3_000).is_err());
    }
}
