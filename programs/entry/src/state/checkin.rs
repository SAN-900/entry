use anchor_lang::prelude::*;

#[account(zero_copy)]
pub struct CheckIns {
    pub event: Pubkey,
    pub supply: u32,
}

impl CheckIns {
    pub const HEADER_SIZE: usize = 8 + core::mem::size_of::<Self>();

    pub const fn account_space(supply: u32) -> usize {
        Self::HEADER_SIZE + (supply as usize + 7) / 8
    }
}

pub fn initialize(checkins: &AccountLoader<CheckIns>, event: Pubkey, supply: u32) -> Result<()> {
    {
        let mut header = checkins.load_init()?;
        header.event = event;
        header.supply = supply;
    }
    let info = checkins.to_account_info();
    let mut data = info.try_borrow_mut_data()?;
    require!(
        data.len() >= CheckIns::account_space(supply),
        crate::errors::EntryError::InvalidSupply
    );
    data[CheckIns::HEADER_SIZE..].fill(0);
    Ok(())
}

pub fn validate(checkins: &AccountLoader<CheckIns>, event: Pubkey, supply: u32) -> Result<()> {
    let header = checkins.load()?;
    require_keys_eq!(
        header.event,
        event,
        crate::errors::EntryError::InvalidCheckInAccount
    );
    require!(
        header.supply == supply,
        crate::errors::EntryError::InvalidCheckInAccount
    );
    require!(
        checkins.to_account_info().data_len() >= CheckIns::account_space(supply),
        crate::errors::EntryError::InvalidCheckInAccount
    );
    Ok(())
}

pub fn is_checked_in(checkins: &AccountLoader<CheckIns>, nonce: u64) -> Result<bool> {
    let info = checkins.to_account_info();
    let data = info.try_borrow_data()?;
    bitmap_contains(&data, nonce)
}

pub fn set_checked_in(checkins: &AccountLoader<CheckIns>, nonce: u64) -> Result<()> {
    let info = checkins.to_account_info();
    let mut data = info.try_borrow_mut_data()?;
    bitmap_set(&mut data, nonce)
}

fn bitmap_contains(data: &[u8], nonce: u64) -> Result<bool> {
    let index = usize::try_from(nonce)
        .map_err(|_| error!(crate::errors::EntryError::TicketIdOutOfRange))?;
    let byte = *data
        .get(CheckIns::HEADER_SIZE + index / 8)
        .ok_or(crate::errors::EntryError::TicketIdOutOfRange)?;
    Ok(byte & (1 << (index % 8)) != 0)
}

fn bitmap_set(data: &mut [u8], nonce: u64) -> Result<()> {
    let index = usize::try_from(nonce)
        .map_err(|_| error!(crate::errors::EntryError::TicketIdOutOfRange))?;
    let byte = data
        .get_mut(CheckIns::HEADER_SIZE + index / 8)
        .ok_or(crate::errors::EntryError::TicketIdOutOfRange)?;
    *byte |= 1 << (index % 8);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitmap_bits_are_independent_and_set_only() {
        let mut data = vec![0; CheckIns::HEADER_SIZE + 2];
        assert!(!bitmap_contains(&data, 0).unwrap());
        bitmap_set(&mut data, 0).unwrap();
        bitmap_set(&mut data, 9).unwrap();
        bitmap_set(&mut data, 0).unwrap();
        assert!(bitmap_contains(&data, 0).unwrap());
        assert!(bitmap_contains(&data, 9).unwrap());
        assert!(!bitmap_contains(&data, 1).unwrap());
    }

    #[test]
    fn bitmap_rejects_indices_outside_allocated_supply() {
        let data = vec![0; CheckIns::HEADER_SIZE + 1];
        assert!(bitmap_contains(&data, 8).is_err());
    }

    #[test]
    fn fifty_thousand_tickets_fit_in_one_bitmap_account() {
        assert_eq!(CheckIns::account_space(50_000), CheckIns::HEADER_SIZE + 6_250);
        assert!(CheckIns::account_space(50_000) < 7_000);
    }
}
