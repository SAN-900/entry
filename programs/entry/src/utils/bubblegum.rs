//! Bubblegum V2 proof-checked ownership transfer for frozen ticket inventory.

use anchor_lang::prelude::*;
use mpl_bubblegum::instructions::{FreezeV2CpiBuilder, ThawV2CpiBuilder, TransferV2CpiBuilder};

use crate::errors::EntryError;

const PERMANENTLY_FROZEN: u8 = 1 << 1;
const NON_TRANSFERABLE: u8 = 1 << 2;
const TICKET_FROZEN_FLAGS: u8 = PERMANENTLY_FROZEN;
const TICKET_THAWED_FLAGS: u8 = 0;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TicketLeafProof {
    pub root: [u8; 32],
    pub data_hash: [u8; 32],
    pub creator_hash: [u8; 32],
    pub asset_data_hash: Option<[u8; 32]>,
    pub flags: Option<u8>,
    pub nonce: u64,
    pub index: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TicketTransferProofs {
    /// Proof against the currently frozen leaf, consumed by thawV2.
    pub thaw: TicketLeafProof,
    /// Proof against the thawed leaf, consumed by transferV2.
    pub transfer: TicketLeafProof,
    /// Proof against the transferred leaf, consumed by freezeV2.
    pub refreeze: TicketLeafProof,
}

impl TicketTransferProofs {
    pub fn validate(&self) -> Result<()> {
        let thaw = &self.thaw;
        let transfer = &self.transfer;
        let refreeze = &self.refreeze;
        require!(
            thaw.data_hash == transfer.data_hash
                && transfer.data_hash == refreeze.data_hash
                && thaw.creator_hash == transfer.creator_hash
                && transfer.creator_hash == refreeze.creator_hash
                && thaw.asset_data_hash == transfer.asset_data_hash
                && transfer.asset_data_hash == refreeze.asset_data_hash
                && thaw.nonce == transfer.nonce
                && transfer.nonce == refreeze.nonce
                && thaw.index == transfer.index
                && transfer.index == refreeze.index,
            EntryError::InvalidBubblegumProof
        );
        require!(
            thaw.flags == Some(TICKET_FROZEN_FLAGS)
                && transfer.flags == Some(TICKET_THAWED_FLAGS)
                && refreeze.flags == Some(TICKET_THAWED_FLAGS),
            EntryError::InvalidBubblegumFlags
        );
        require!(
            thaw.flags.unwrap() & NON_TRANSFERABLE == 0
                && transfer.flags.unwrap() & NON_TRANSFERABLE == 0
                && refreeze.flags.unwrap() & NON_TRANSFERABLE == 0,
            EntryError::InvalidBubblegumFlags
        );
        Ok(())
    }
}

/// Atomically thaws a collection-frozen ticket, transfers it under ENTRY's permanent transfer
/// delegate, and refreezes it under ENTRY's permanent freeze delegate. Any failed CPI reverts
/// the complete transaction, including token transfers performed by the caller.
#[allow(clippy::too_many_arguments)]
pub fn transfer_frozen_ticket<'info>(
    bubblegum_program: &AccountInfo<'info>,
    tree_config: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    old_owner: &AccountInfo<'info>,
    new_owner: &AccountInfo<'info>,
    event_authority: &AccountInfo<'info>,
    merkle_tree: &AccountInfo<'info>,
    core_collection: &AccountInfo<'info>,
    log_wrapper: &AccountInfo<'info>,
    compression_program: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    proofs: &TicketTransferProofs,
    proof_accounts: &[AccountInfo<'info>],
    event_seeds: &[&[u8]],
) -> Result<()> {
    proofs.validate()?;
    require!(
        proof_accounts.len() <= 32
            && proof_accounts
                .iter()
                .all(|account| !account.is_signer && !account.is_writable),
        EntryError::InvalidMerkleProofAccounts
    );
    let remaining: Vec<_> = proof_accounts
        .iter()
        .map(|account| (account, false, false))
        .collect();

    let mut thaw = ThawV2CpiBuilder::new(bubblegum_program);
    thaw.tree_config(tree_config)
        .payer(payer)
        .authority(Some(event_authority))
        .leaf_owner(old_owner)
        .leaf_delegate(old_owner)
        .merkle_tree(merkle_tree)
        .core_collection(Some(core_collection))
        .log_wrapper(log_wrapper)
        .compression_program(compression_program)
        .system_program(system_program)
        .root(proofs.thaw.root)
        .data_hash(proofs.thaw.data_hash)
        .creator_hash(proofs.thaw.creator_hash)
        .nonce(proofs.thaw.nonce)
        .index(proofs.thaw.index)
        .add_remaining_accounts(&remaining);
    if let Some(hash) = proofs.thaw.asset_data_hash {
        thaw.asset_data_hash(hash);
    }
    thaw.flags(proofs.thaw.flags.unwrap())
        .invoke_signed(&[event_seeds])?;

    let mut transfer = TransferV2CpiBuilder::new(bubblegum_program);
    transfer
        .tree_config(tree_config)
        .payer(payer)
        .authority(Some(event_authority))
        .leaf_owner(old_owner)
        .leaf_delegate(Some(old_owner))
        .new_leaf_owner(new_owner)
        .merkle_tree(merkle_tree)
        .core_collection(Some(core_collection))
        .log_wrapper(log_wrapper)
        .compression_program(compression_program)
        .system_program(system_program)
        .root(proofs.transfer.root)
        .data_hash(proofs.transfer.data_hash)
        .creator_hash(proofs.transfer.creator_hash)
        .nonce(proofs.transfer.nonce)
        .index(proofs.transfer.index)
        .add_remaining_accounts(&remaining);
    if let Some(hash) = proofs.transfer.asset_data_hash {
        transfer.asset_data_hash(hash);
    }
    transfer
        .flags(proofs.transfer.flags.unwrap())
        .invoke_signed(&[event_seeds])?;

    let mut freeze = FreezeV2CpiBuilder::new(bubblegum_program);
    freeze
        .tree_config(tree_config)
        .payer(payer)
        .authority(Some(event_authority))
        .leaf_owner(new_owner)
        .leaf_delegate(new_owner)
        .merkle_tree(merkle_tree)
        .core_collection(Some(core_collection))
        .log_wrapper(log_wrapper)
        .compression_program(compression_program)
        .system_program(system_program)
        .root(proofs.refreeze.root)
        .data_hash(proofs.refreeze.data_hash)
        .creator_hash(proofs.refreeze.creator_hash)
        .nonce(proofs.refreeze.nonce)
        .index(proofs.refreeze.index)
        .add_remaining_accounts(&remaining);
    if let Some(hash) = proofs.refreeze.asset_data_hash {
        freeze.asset_data_hash(hash);
    }
    freeze
        .flags(proofs.refreeze.flags.unwrap())
        .invoke_signed(&[event_seeds])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proof(flags: u8) -> TicketLeafProof {
        TicketLeafProof {
            root: [1; 32],
            data_hash: [2; 32],
            creator_hash: [3; 32],
            asset_data_hash: None,
            flags: Some(flags),
            nonce: 7,
            index: 7,
        }
    }

    fn valid_transfer_proofs() -> TicketTransferProofs {
        TicketTransferProofs {
            thaw: proof(TICKET_FROZEN_FLAGS),
            transfer: proof(TICKET_THAWED_FLAGS),
            refreeze: proof(TICKET_THAWED_FLAGS),
        }
    }

    #[test]
    fn transfer_requires_frozen_then_thawed_leaf_states() {
        valid_transfer_proofs().validate().unwrap();

        let mut invalid = valid_transfer_proofs();
        invalid.thaw.flags = Some(TICKET_THAWED_FLAGS);
        assert!(invalid.validate().is_err());

        let mut invalid = valid_transfer_proofs();
        invalid.refreeze.flags = Some(TICKET_FROZEN_FLAGS);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn transfer_rejects_nontransferable_assets_and_mismatched_hashes() {
        let mut nontransferable = valid_transfer_proofs();
        nontransferable.transfer.flags = Some(NON_TRANSFERABLE);
        assert!(nontransferable.validate().is_err());

        let mut mismatched = valid_transfer_proofs();
        mismatched.refreeze.data_hash = [9; 32];
        assert!(mismatched.validate().is_err());
    }

    #[test]
    fn transfer_requires_one_nonce_and_leaf_index_across_proofs() {
        let mut mismatched_nonce = valid_transfer_proofs();
        mismatched_nonce.transfer.nonce += 1;
        assert!(mismatched_nonce.validate().is_err());

        let mut mismatched_index = valid_transfer_proofs();
        mismatched_index.refreeze.index += 1;
        assert!(mismatched_index.validate().is_err());
    }
}
