//! ENTRY's compressed owner-of-record tree.
//!
//! The SPL Account Compression program stores the tree. ENTRY owns its
//! authority through the Event PDA and changes only leaf `nonce` after a
//! successful purchase or protocol resale. The root is mirrored in Event so
//! scanners can verify current ownership without trusting NFT metadata.

use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program::invoke_signed,
    },
};
use borsh::BorshSerialize;

use crate::{constants::ACCOUNT_COMPRESSION_PROGRAM_ID, errors::EntryError};

const LEAF_DOMAIN: &[u8] = b"ENTRY_OWNER_V1";
pub const REGISTRY_BUFFER_SIZE: u32 = 64;
const CMT_HEADER_SIZE: usize = 56;

fn cmt_instruction_data(name: &[u8], args: &impl BorshSerialize) -> Result<Vec<u8>> {
    let discriminator = solana_sha256_hasher::hash(name).to_bytes();
    let mut data = discriminator[..8].to_vec();
    args.serialize(&mut data)?;
    Ok(data)
}

fn tree_size(depth: u8) -> Result<usize> {
    let tree_bytes = match depth {
        14 => core::mem::size_of::<spl_account_compression::ConcurrentMerkleTree<14, 64>>(),
        20 => core::mem::size_of::<spl_account_compression::ConcurrentMerkleTree<20, 64>>(),
        _ => return err!(EntryError::InvalidRegistryTree),
    };
    CMT_HEADER_SIZE
        .checked_add(tree_bytes)
        .ok_or_else(|| error!(EntryError::ArithmeticOverflow))
}

fn validate_tree_header(data: &[u8], depth: u8, authority: Pubkey) -> Result<()> {
    require!(
        data.len() >= tree_size(depth)?,
        EntryError::InvalidRegistryTree
    );
    // SPL Account Compression v1 stores a one-byte account type, a one-byte
    // header variant, then the Borsh v1 fields: buffer, depth, authority.
    require!(
        data[0] == 1
            && data[1] == 0
            && u32::from_le_bytes(data[2..6].try_into().unwrap()) == REGISTRY_BUFFER_SIZE
            && u32::from_le_bytes(data[6..10].try_into().unwrap()) == u32::from(depth)
            && data[10..42] == authority.to_bytes(),
        EntryError::InvalidRegistryTree
    );
    Ok(())
}

/// Verify a Bubblegum tree's SPL Account Compression header and return its
/// supported depth. The tree-config PDA is the CMT authority for Bubblegum.
pub fn validate_bubblegum_tree(data: &[u8], tree_config: Pubkey) -> Result<u8> {
    require!(data.len() >= CMT_HEADER_SIZE, EntryError::InvalidEventTree);
    require!(
        u32::from_le_bytes(data[2..6].try_into().unwrap()) == REGISTRY_BUFFER_SIZE,
        EntryError::InvalidEventTree
    );
    let depth = u32::from_le_bytes(data[6..10].try_into().unwrap());
    let depth = u8::try_from(depth).map_err(|_| error!(EntryError::InvalidEventTree))?;
    require!(depth == 14 || depth == 20, EntryError::InvalidEventTree);
    validate_tree_header(data, depth, tree_config)
        .map_err(|_| error!(EntryError::InvalidEventTree))?;
    Ok(depth)
}

/// Hash an event ticket's current owner record. The nonce is fixed to its
/// Bubblegum leaf index, and the distinct domain prevents cross-protocol use.
pub fn owner_leaf(event: Pubkey, nonce: u64, owner: Pubkey) -> [u8; 32] {
    hashv(&[
        LEAF_DOMAIN,
        event.as_ref(),
        &nonce.to_le_bytes(),
        owner.as_ref(),
    ])
}

/// SPL Account Compression's empty leaf is the all-zero node.
pub const EMPTY_LEAF: [u8; 32] = [0; 32];

fn hashv(parts: &[&[u8]]) -> [u8; 32] {
    // Match SPL ConcurrentMerkleTree's Keccak node hashing.
    solana_keccak_hasher::hashv(parts).to_bytes()
}

pub fn empty_root(depth: u8) -> [u8; 32] {
    let mut node = EMPTY_LEAF;
    for _ in 0..depth {
        node = hashv(&[&node, &node]);
    }
    node
}

/// Initialize a preallocated, zeroed SPL ConcurrentMerkleTree with the Event
/// PDA as its sole authority. The client only allocates the compression-owned
/// account; it cannot choose the root or retain tree authority.
pub fn initialize_tree<'info>(
    compression_program: &AccountInfo<'info>,
    registry_tree: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    log_wrapper: &AccountInfo<'info>,
    depth: u8,
    signer_seeds: &[&[u8]],
) -> Result<()> {
    require_keys_eq!(
        *compression_program.key,
        ACCOUNT_COMPRESSION_PROGRAM_ID,
        EntryError::InvalidRegistryTree
    );
    require_keys_eq!(
        *registry_tree.owner,
        ACCOUNT_COMPRESSION_PROGRAM_ID,
        EntryError::InvalidRegistryTree
    );
    require_keys_eq!(
        *log_wrapper.key,
        crate::constants::NOOP_PROGRAM_ID,
        EntryError::InvalidRegistryTree
    );
    {
        let data = registry_tree.try_borrow_data()?;
        require!(
            data.len() >= tree_size(depth)?,
            EntryError::InvalidRegistryTree
        );
        require!(
            data.iter().all(|byte| *byte == 0),
            EntryError::InvalidRegistryTree
        );
    }

    let data = cmt_instruction_data(
        b"global:init_empty_merkle_tree",
        &spl_account_compression::instruction::InitEmptyMerkleTree {
            max_depth: u32::from(depth),
            max_buffer_size: REGISTRY_BUFFER_SIZE,
        },
    )?;
    // SPL Initialize account order is merkle_tree, authority, noop.
    let instruction = Instruction {
        program_id: ACCOUNT_COMPRESSION_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*registry_tree.key, false),
            AccountMeta::new_readonly(*authority.key, true),
            AccountMeta::new_readonly(*log_wrapper.key, false),
        ],
        data,
    };
    invoke_signed(
        &instruction,
        &[
            registry_tree.clone(),
            authority.clone(),
            log_wrapper.clone(),
            compression_program.clone(),
        ],
        &[signer_seeds],
    )?;
    validate_tree_header(&registry_tree.try_borrow_data()?, depth, *authority.key)?;
    Ok(())
}

fn compute_root(nonce: u64, proof: &[[u8; 32]], leaf: [u8; 32]) -> [u8; 32] {
    let mut computed = leaf;
    for (level, sibling) in proof.iter().enumerate() {
        computed = if ((nonce >> level) & 1) == 0 {
            hashv(&[&computed, sibling])
        } else {
            hashv(&[sibling, &computed])
        };
    }
    computed
}

pub fn root_from_proof(
    root: [u8; 32],
    nonce: u64,
    proof: &[AccountInfo<'_>],
    old_leaf: [u8; 32],
) -> Result<[u8; 32]> {
    require!(proof.len() <= 30, EntryError::InvalidRegistryProof);
    let siblings: Vec<[u8; 32]> = proof
        .iter()
        .map(|account| account.key().to_bytes())
        .collect();
    let computed = compute_root(nonce, &siblings, old_leaf);
    require!(computed == root, EntryError::InvalidRegistryProof);
    Ok(computed)
}

/// Replace one owner leaf in the SPL compressed tree. `proof` is the first
/// `Event.registry_depth` slice of the instruction's remaining accounts.
pub fn replace_owner_leaf<'info>(
    compression_program: &AccountInfo<'info>,
    registry_tree: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    log_wrapper: &AccountInfo<'info>,
    nonce: u64,
    old_leaf: [u8; 32],
    new_leaf: [u8; 32],
    root: [u8; 32],
    proof: &[AccountInfo<'info>],
    signer_seeds: &[&[u8]],
) -> Result<[u8; 32]> {
    require_keys_eq!(
        *compression_program.key,
        ACCOUNT_COMPRESSION_PROGRAM_ID,
        EntryError::InvalidRegistryTree
    );
    require_keys_eq!(
        *registry_tree.owner,
        ACCOUNT_COMPRESSION_PROGRAM_ID,
        EntryError::InvalidRegistryTree
    );
    require_keys_eq!(
        *log_wrapper.key,
        crate::constants::NOOP_PROGRAM_ID,
        EntryError::InvalidRegistryTree
    );
    require!(
        proof.len() <= 30
            && proof
                .iter()
                .all(|account| !account.is_signer && !account.is_writable),
        EntryError::InvalidRegistryProof
    );
    validate_tree_header(
        &registry_tree.try_borrow_data()?,
        u8::try_from(proof.len()).map_err(|_| error!(EntryError::InvalidRegistryProof))?,
        *authority.key,
    )?;
    require!(
        nonce < (1u64 << proof.len()),
        EntryError::InvalidRegistryProof
    );

    root_from_proof(root, nonce, proof, old_leaf)?;
    let index = u32::try_from(nonce).map_err(|_| error!(EntryError::ArithmeticOverflow))?;
    let data = cmt_instruction_data(
        b"global:replace_leaf",
        &spl_account_compression::instruction::ReplaceLeaf {
            root,
            previous_leaf: old_leaf,
            new_leaf,
            index,
        },
    )?;

    // The SPL `Modify` account order is merkle_tree, authority, noop.
    let mut accounts = vec![
        AccountMeta::new(*registry_tree.key, false),
        AccountMeta::new_readonly(*authority.key, true),
        AccountMeta::new_readonly(*log_wrapper.key, false),
    ];
    accounts.extend(
        proof
            .iter()
            .map(|account| AccountMeta::new_readonly(account.key(), false)),
    );
    let instruction = Instruction {
        program_id: ACCOUNT_COMPRESSION_PROGRAM_ID,
        accounts,
        data,
    };
    let mut account_infos = vec![
        registry_tree.clone(),
        authority.clone(),
        log_wrapper.clone(),
    ];
    account_infos.extend(proof.iter().cloned());
    account_infos.push(compression_program.clone());
    invoke_signed(&instruction, &account_infos, &[signer_seeds])?;
    let siblings: Vec<[u8; 32]> = proof
        .iter()
        .map(|account| account.key().to_bytes())
        .collect();
    Ok(compute_root(nonce, &siblings, new_leaf))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fold(nonce: u64, siblings: &[[u8; 32]], leaf: [u8; 32]) -> [u8; 32] {
        compute_root(nonce, siblings, leaf)
    }

    #[test]
    fn owner_leaf_is_event_nonce_and_owner_bound() {
        let event = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        assert_ne!(owner_leaf(event, 4, owner), owner_leaf(event, 5, owner));
        assert_ne!(
            owner_leaf(event, 4, owner),
            owner_leaf(Pubkey::new_unique(), 4, owner)
        );
        assert_ne!(
            owner_leaf(event, 4, owner),
            owner_leaf(event, 4, Pubkey::new_unique())
        );
    }

    #[test]
    fn proof_reconstructs_root_and_replacement_changes_only_that_leaf() {
        let siblings = [[1; 32], [2; 32], [3; 32]];
        let old = owner_leaf(Pubkey::new_unique(), 5, Pubkey::new_unique());
        let new = owner_leaf(Pubkey::new_unique(), 5, Pubkey::new_unique());
        let root = fold(5, &siblings, old);
        assert_eq!(fold(5, &siblings, old), root);
        assert_ne!(fold(5, &siblings, EMPTY_LEAF), root);
        assert_ne!(fold(5, &siblings, new), root);
    }

    #[test]
    fn empty_leaf_is_not_a_valid_ticket_owner_leaf() {
        assert_ne!(
            EMPTY_LEAF,
            owner_leaf(Pubkey::new_unique(), 0, Pubkey::new_unique())
        );
    }

    #[test]
    fn registry_tree_size_accepts_only_supported_depths() {
        assert!(tree_size(14).unwrap() > CMT_HEADER_SIZE);
        assert!(tree_size(20).unwrap() > tree_size(14).unwrap());
        assert!(tree_size(13).is_err());
    }

    #[test]
    fn registry_header_binds_depth_buffer_and_event_authority() {
        let event = Pubkey::new_unique();
        let mut data = vec![0; tree_size(14).unwrap()];
        data[0] = 1; // ConcurrentMerkleTree account type.
        data[1] = 0; // V1 header variant.
        data[2..6].copy_from_slice(&REGISTRY_BUFFER_SIZE.to_le_bytes());
        data[6..10].copy_from_slice(&14u32.to_le_bytes());
        data[10..42].copy_from_slice(event.as_ref());
        validate_tree_header(&data, 14, event).unwrap();
        assert!(validate_tree_header(&data, 20, event).is_err());
        assert!(validate_tree_header(&data, 14, Pubkey::new_unique()).is_err());

        data[2..6].copy_from_slice(&32u32.to_le_bytes());
        assert!(validate_tree_header(&data, 14, event).is_err());
    }

    #[test]
    fn bubblegum_tree_requires_supported_depth_buffer_and_tree_config_authority() {
        let tree_config = Pubkey::new_unique();
        let mut data = vec![0; tree_size(14).unwrap()];
        data[0] = 1;
        data[1] = 0;
        data[2..6].copy_from_slice(&REGISTRY_BUFFER_SIZE.to_le_bytes());
        data[6..10].copy_from_slice(&14u32.to_le_bytes());
        data[10..42].copy_from_slice(tree_config.as_ref());
        assert_eq!(validate_bubblegum_tree(&data, tree_config).unwrap(), 14);

        data[2..6].copy_from_slice(&32u32.to_le_bytes());
        assert!(validate_bubblegum_tree(&data, tree_config).is_err());
        data[2..6].copy_from_slice(&REGISTRY_BUFFER_SIZE.to_le_bytes());
        assert!(validate_bubblegum_tree(&data, Pubkey::new_unique()).is_err());
    }
}
