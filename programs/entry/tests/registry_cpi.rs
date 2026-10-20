//! Runtime CPI test. Build `entry.so` and `spl_account_compression.so`, place
//! both in `BPF_OUT_DIR`, then run this ignored test explicitly. The test uses
//! SBF for ENTRY and the real SPL Account Compression program.

use anchor_lang::solana_program::{
    program_error::{ProgramError, ProgramResult},
    program_pack::Pack,
};
use anchor_lang::{
    AccountDeserialize, AccountSerialize, AnchorSerialize, InstructionData, ToAccountMetas,
};
use anchor_spl::token::spl_token::{
    self,
    solana_program::program_option::COption,
    state::{Account as SplTokenAccount, AccountState, Mint as SplMint},
};
use borsh::BorshSerialize;
use entry::{
    accounts, instruction,
    state::{Config, Event},
    ID as ENTRY_ID,
};
use mpl_bubblegum::{
    accounts::TreeConfig,
    types::{DecompressibleState, Version},
};
use mpl_core::{
    accounts::{BaseCollectionV1, PluginHeaderV1, PluginRegistryV1},
    types::{
        BubblegumV2, Key, PermanentFreezeDelegate, PermanentTransferDelegate, Plugin,
        PluginAuthority, PluginType, RegistryRecord,
    },
};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{
    account::Account,
    compute_budget::ComputeBudgetInstruction,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    rent::Rent,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

const MPL_CORE_CPI_SIGNER: Pubkey =
    solana_sdk::pubkey!("CbNY3JiXdXNE9tPNEk1aRZVEkWdj2v7kfJLNQwZZgpXk");

fn mock_bubblegum(
    _program_id: &Pubkey,
    _accounts: &[anchor_lang::solana_program::account_info::AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // This processor only accepts expected V2 CPI discriminators. It does not
    // execute Bubblegum state transitions, so it cannot prove leaf rules.
    let discriminator = instruction_data
        .get(..8)
        .ok_or(ProgramError::InvalidInstructionData)?;
    if discriminator == [120, 121, 23, 146, 173, 110, 199, 205]
        || discriminator == [200, 151, 244, 102, 16, 195, 255, 3]
        || discriminator == [96, 133, 101, 93, 82, 220, 146, 191]
        || discriminator == [119, 40, 6, 235, 234, 221, 248, 49]
    {
        Ok(())
    } else {
        Err(ProgramError::InvalidInstructionData)
    }
}

fn token_program_account(lamports: u64, data: Vec<u8>) -> Account {
    Account {
        lamports,
        data,
        owner: spl_token::ID,
        executable: false,
        rent_epoch: 0,
    }
}

fn initialized_mint(decimals: u8) -> Account {
    let mut data = vec![0; SplMint::LEN];
    SplMint {
        mint_authority: COption::None,
        supply: 10_000_000_000,
        decimals,
        is_initialized: true,
        freeze_authority: COption::None,
    }
    .pack_into_slice(&mut data);
    token_program_account(Rent::default().minimum_balance(SplMint::LEN), data)
}

fn initialized_token_account(mint: Pubkey, owner: Pubkey, amount: u64) -> Account {
    let mut data = vec![0; SplTokenAccount::LEN];
    SplTokenAccount {
        mint,
        owner,
        amount,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    }
    .pack_into_slice(&mut data);
    token_program_account(Rent::default().minimum_balance(SplTokenAccount::LEN), data)
}

fn token_amount(account: &Account) -> u64 {
    SplTokenAccount::unpack(&account.data).unwrap().amount
}

fn compression_instruction_data(name: &[u8], args: &impl BorshSerialize) -> Vec<u8> {
    let hash = solana_sha256_hasher::hash(name).to_bytes();
    let mut data = hash[..8].to_vec();
    args.serialize(&mut data).unwrap();
    data
}

fn event_controlled_core_collection(event: Pubkey) -> Vec<u8> {
    let base = BaseCollectionV1 {
        key: Key::CollectionV1,
        update_authority: event,
        name: "ENTRY collection".to_owned(),
        uri: "https://example.invalid".to_owned(),
        num_minted: 0,
        current_size: 0,
    };
    let base_bytes = base.try_to_vec().unwrap();
    let plugins = [
        (
            PluginType::PermanentFreezeDelegate,
            Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: false }),
        ),
        (
            PluginType::PermanentTransferDelegate,
            Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
        ),
        (PluginType::BubblegumV2, Plugin::BubblegumV2(BubblegumV2 {})),
    ];
    let mut plugin_bytes = Vec::new();
    let mut registry = Vec::new();
    for (plugin_type, plugin) in plugins {
        let offset = base_bytes.len() + PluginHeaderV1::LEN + plugin_bytes.len();
        let bytes = plugin.try_to_vec().unwrap();
        plugin_bytes.extend_from_slice(&bytes);
        registry.push(RegistryRecord {
            plugin_type,
            authority: PluginAuthority::UpdateAuthority,
            offset: offset as u64,
        });
    }
    let registry_offset = base_bytes.len() + PluginHeaderV1::LEN + plugin_bytes.len();
    let header = PluginHeaderV1 {
        key: Key::PluginHeaderV1,
        plugin_registry_offset: registry_offset as u64,
    };
    let plugin_registry = PluginRegistryV1 {
        key: Key::PluginRegistryV1,
        registry,
        external_registry: Vec::new(),
    };
    let mut data = base_bytes;
    data.extend_from_slice(&header.try_to_vec().unwrap());
    data.extend_from_slice(&plugin_bytes);
    data.extend_from_slice(&plugin_registry.try_to_vec().unwrap());
    data
}

#[tokio::test]
#[ignore = "requires ENTRY and SPL Account Compression SBF programs in BPF_OUT_DIR; Bubblegum CPI is mocked"]
async fn purchase_updates_registry_and_failed_proof_rolls_back_usdc() {
    let bpf_dir = std::env::var("BPF_OUT_DIR").expect("set BPF_OUT_DIR to the SBF artifacts");
    assert!(std::path::Path::new(&bpf_dir).join("entry.so").exists());
    assert!(std::path::Path::new(&bpf_dir)
        .join("spl_account_compression.so")
        .exists());

    let organizer = Keypair::new();
    let buyer = Keypair::new();
    let resale_buyer = Keypair::new();
    let gatekeeper = Keypair::new();
    let fee_vault = Pubkey::new_unique();
    let event_id = 901u64;
    let event = Pubkey::find_program_address(
        &[
            b"event",
            organizer.pubkey().as_ref(),
            &event_id.to_le_bytes(),
        ],
        &ENTRY_ID,
    )
    .0;
    let ticket_tree = Pubkey::new_unique();
    let registry_tree = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    let payment_mint = entry::constants::USDC_DEVNET_MINT;
    let (tree_config_key, _) = TreeConfig::find_pda(&ticket_tree);
    let tree_config = TreeConfig {
        discriminator: [0; 8],
        tree_creator: organizer.pubkey(),
        tree_delegate: event,
        total_mint_capacity: 16_384,
        num_minted: 0,
        is_public: false,
        is_decompressible: DecompressibleState::Disabled,
        version: Version::V2,
    };
    let mut tree_config_data = Vec::new();
    tree_config.serialize(&mut tree_config_data).unwrap();
    let ticket_tree_space =
        56 + core::mem::size_of::<spl_account_compression::ConcurrentMerkleTree<14, 64>>();
    let mut ticket_tree_data = vec![0; ticket_tree_space];
    ticket_tree_data[0] = 1;
    ticket_tree_data[1] = 0;
    ticket_tree_data[2..6].copy_from_slice(&64u32.to_le_bytes());
    ticket_tree_data[6..10].copy_from_slice(&14u32.to_le_bytes());
    ticket_tree_data[10..42].copy_from_slice(tree_config_key.as_ref());

    let (config_key, config_bump) = Pubkey::find_program_address(&[b"config"], &ENTRY_ID);
    let config = Config {
        admin: organizer.pubkey(),
        fee_vault,
        fee_bps: 500,
        bump: config_bump,
    };
    let mut config_data = Vec::new();
    config.try_serialize(&mut config_data).unwrap();
    let buyer_usdc = anchor_spl::associated_token::get_associated_token_address_with_program_id(
        &buyer.pubkey(),
        &payment_mint,
        &spl_token::ID,
    );
    let resale_buyer_usdc =
        anchor_spl::associated_token::get_associated_token_address_with_program_id(
            &resale_buyer.pubkey(),
            &payment_mint,
            &spl_token::ID,
        );
    let organizer_usdc = anchor_spl::associated_token::get_associated_token_address_with_program_id(
        &organizer.pubkey(),
        &payment_mint,
        &spl_token::ID,
    );
    let fee_vault_usdc = anchor_spl::associated_token::get_associated_token_address_with_program_id(
        &fee_vault,
        &payment_mint,
        &spl_token::ID,
    );

    let mut test = ProgramTest::new("entry", ENTRY_ID, None);
    test.add_account(
        organizer.pubkey(),
        Account {
            lamports: 10_000_000_000,
            data: Vec::new(),
            owner: anchor_lang::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(
        buyer.pubkey(),
        Account {
            lamports: 10_000_000_000,
            data: Vec::new(),
            owner: anchor_lang::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(
        resale_buyer.pubkey(),
        Account {
            lamports: 10_000_000_000,
            data: Vec::new(),
            owner: anchor_lang::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(
        gatekeeper.pubkey(),
        Account {
            lamports: 1_000_000,
            data: Vec::new(),
            owner: anchor_lang::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(
        config_key,
        Account {
            lamports: Rent::default().minimum_balance(config_data.len()),
            data: config_data,
            owner: ENTRY_ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(payment_mint, initialized_mint(6));
    test.add_account(
        buyer_usdc,
        initialized_token_account(payment_mint, buyer.pubkey(), 2_000_000),
    );
    test.add_account(
        resale_buyer_usdc,
        initialized_token_account(payment_mint, resale_buyer.pubkey(), 2_000_000),
    );
    test.add_account(
        organizer_usdc,
        initialized_token_account(payment_mint, organizer.pubkey(), 0),
    );
    test.add_account(
        fee_vault_usdc,
        initialized_token_account(payment_mint, fee_vault, 0),
    );
    test.add_account(
        ticket_tree,
        Account {
            lamports: 10_000_000,
            data: ticket_tree_data,
            owner: entry::constants::ACCOUNT_COMPRESSION_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(
        tree_config_key,
        Account {
            lamports: 10_000_000,
            data: tree_config_data,
            owner: mpl_bubblegum::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(
        collection,
        Account {
            lamports: 10_000_000,
            data: event_controlled_core_collection(event),
            owner: entry::constants::MPL_CORE_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_program(
        "spl_account_compression",
        entry::constants::ACCOUNT_COMPRESSION_PROGRAM_ID,
        None,
    );
    test.prefer_bpf(false);
    test.add_program(
        "spl_noop",
        entry::constants::NOOP_PROGRAM_ID,
        processor!(spl_noop::noop),
    );
    test.add_program(
        "mock_bubblegum",
        mpl_bubblegum::ID,
        processor!(mock_bubblegum),
    );
    test.add_program(
        "spl_token",
        spl_token::ID,
        processor!(spl_token::processor::Processor::process),
    );
    // Purchase requires these fixed program accounts; only Bubblegum is
    // invoked in the purchase path, and the mock intentionally does not CPI.
    test.add_account(
        entry::constants::MPL_CORE_PROGRAM_ID,
        Account {
            lamports: 1,
            data: Vec::new(),
            owner: Pubkey::default(),
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(
        MPL_CORE_CPI_SIGNER,
        Account {
            lamports: 1,
            data: Vec::new(),
            owner: Pubkey::default(),
            executable: false,
            rent_epoch: 0,
        },
    );
    let registry_space =
        56 + core::mem::size_of::<spl_account_compression::ConcurrentMerkleTree<14, 64>>();
    test.add_account(
        registry_tree,
        Account {
            lamports: Rent::default().minimum_balance(registry_space) + 1,
            data: vec![0; registry_space],
            owner: entry::constants::ACCOUNT_COMPRESSION_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    let mut context = test.start_with_context().await;

    let event_args = entry::instructions::create_event::CreateEventArgs {
        event_id,
        merkle_tree: ticket_tree,
        registry_tree,
        registry_depth: 14,
        collection,
        payment_mint,
        gate_authority: Pubkey::new_unique(),
        gatekeeper: gatekeeper.pubkey(),
        price: 1_000_000,
        supply: 1,
        max_per_wallet: 1,
        resale_cap_bps: 11_000,
        royalty_bps: 500,
        ticket_name: "ENTRY".to_owned(),
        ticket_symbol: "ENT".to_owned(),
        ticket_uri: "https://example.invalid/tickets".to_owned(),
    };
    let checkins = Pubkey::find_program_address(&[b"checkins", event.as_ref()], &ENTRY_ID).0;
    let create_event = Instruction {
        program_id: ENTRY_ID,
        accounts: accounts::CreateEvent {
            organizer: organizer.pubkey(),
            event,
            checkins,
            merkle_tree: ticket_tree,
            registry_tree,
            tree_config: tree_config_key,
            collection,
            bubblegum_program: mpl_bubblegum::ID,
            mpl_core_program: entry::constants::MPL_CORE_PROGRAM_ID,
            compression_program: entry::constants::ACCOUNT_COMPRESSION_PROGRAM_ID,
            log_wrapper: entry::constants::NOOP_PROGRAM_ID,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::CreateEvent { args: event_args }.data(),
    };
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(400_000),
            create_event,
        ],
        Some(&organizer.pubkey()),
        &[&organizer],
        blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let depth = 14usize;
    let mut proof_hashes = Vec::with_capacity(depth);
    let mut empty_node = [0u8; 32];
    for _ in 0..depth {
        proof_hashes.push(empty_node);
        empty_node = solana_keccak_hasher::hashv(&[&empty_node, &empty_node]).to_bytes();
    }
    let mut proof_accounts: Vec<_> = proof_hashes
        .iter()
        .map(|hash| AccountMeta::new_readonly(Pubkey::new_from_array(*hash), false))
        .collect();
    let purchase_instruction = |proof: Vec<AccountMeta>| {
        let mut accounts = accounts::Purchase {
            buyer: buyer.pubkey(),
            gatekeeper: gatekeeper.pubkey(),
            event,
            purchase_counter: Pubkey::find_program_address(
                &[b"purchases", event.as_ref(), buyer.pubkey().as_ref()],
                &ENTRY_ID,
            )
            .0,
            config: config_key,
            payment_mint,
            buyer_usdc,
            organizer_usdc,
            fee_vault_usdc,
            tree_config: tree_config_key,
            merkle_tree: ticket_tree,
            collection,
            registry_tree,
            mpl_core_cpi_signer: MPL_CORE_CPI_SIGNER,
            bubblegum_program: mpl_bubblegum::ID,
            compression_program: entry::constants::ACCOUNT_COMPRESSION_PROGRAM_ID,
            log_wrapper: entry::constants::NOOP_PROGRAM_ID,
            mpl_core_program: entry::constants::MPL_CORE_PROGRAM_ID,
            token_program: spl_token::ID,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None);
        accounts.extend(proof);
        Instruction {
            program_id: ENTRY_ID,
            accounts,
            data: instruction::Purchase {
                args: entry::instructions::purchase::PurchaseArgs {
                    expected_nonce: 0,
                    freeze_root: [7; 32],
                },
            }
            .data(),
        }
    };

    let mut invalid_proof = proof_accounts.clone();
    invalid_proof[0] = AccountMeta::new_readonly(Pubkey::new_unique(), false);
    let invalid_purchase = purchase_instruction(invalid_proof);
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(1_000_000),
            invalid_purchase,
        ],
        Some(&buyer.pubkey()),
        &[&buyer, &gatekeeper],
        blockhash,
    );
    assert!(context
        .banks_client
        .process_transaction(transaction)
        .await
        .is_err());
    assert_eq!(
        token_amount(
            &context
                .banks_client
                .get_account(buyer_usdc)
                .await
                .unwrap()
                .unwrap()
        ),
        2_000_000
    );
    assert_eq!(
        token_amount(
            &context
                .banks_client
                .get_account(organizer_usdc)
                .await
                .unwrap()
                .unwrap()
        ),
        0
    );
    assert_eq!(
        token_amount(
            &context
                .banks_client
                .get_account(fee_vault_usdc)
                .await
                .unwrap()
                .unwrap()
        ),
        0
    );
    let event_account = context
        .banks_client
        .get_account(event)
        .await
        .unwrap()
        .unwrap();
    let event_state = Event::try_deserialize(&mut event_account.data.as_slice()).unwrap();
    assert_eq!(event_state.sold, 0);
    assert_eq!(event_state.registry_seq, 0);

    let successful_purchase = purchase_instruction(core::mem::take(&mut proof_accounts));
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(1_000_000),
            successful_purchase,
        ],
        Some(&buyer.pubkey()),
        &[&buyer, &gatekeeper],
        blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
    assert_eq!(
        token_amount(
            &context
                .banks_client
                .get_account(buyer_usdc)
                .await
                .unwrap()
                .unwrap()
        ),
        1_000_000
    );
    assert_eq!(
        token_amount(
            &context
                .banks_client
                .get_account(organizer_usdc)
                .await
                .unwrap()
                .unwrap()
        ),
        950_000
    );
    assert_eq!(
        token_amount(
            &context
                .banks_client
                .get_account(fee_vault_usdc)
                .await
                .unwrap()
                .unwrap()
        ),
        50_000
    );
    let event_account = context
        .banks_client
        .get_account(event)
        .await
        .unwrap()
        .unwrap();
    let event_state = Event::try_deserialize(&mut event_account.data.as_slice()).unwrap();
    assert_eq!(event_state.sold, 1);
    assert_eq!(event_state.registry_seq, 1);
    let expected_root = {
        let leaf = entry::utils::registry::owner_leaf(event, 0, buyer.pubkey());
        let mut root = leaf;
        for sibling in &proof_hashes {
            root = solana_keccak_hasher::hashv(&[&root, sibling]).to_bytes();
        }
        root
    };
    assert_eq!(event_state.registry_root, expected_root);
    let tree = context
        .banks_client
        .get_account(registry_tree)
        .await
        .unwrap()
        .unwrap();
    let tree_state: &spl_account_compression::ConcurrentMerkleTree<14, 64> =
        bytemuck::from_bytes(&tree.data[56..]);
    assert_eq!(tree_state.get_root(), expected_root);
    assert_eq!(tree_state.sequence_number, 1);

    let asset_id = mpl_bubblegum::utils::get_asset_id(&ticket_tree, 0);
    let listing = Pubkey::find_program_address(
        &[
            b"listing",
            event.as_ref(),
            asset_id.as_ref(),
            buyer.pubkey().as_ref(),
        ],
        &ENTRY_ID,
    )
    .0;
    let over_cap_listing = Instruction {
        program_id: ENTRY_ID,
        accounts: accounts::ListForResale {
            seller: buyer.pubkey(),
            event,
            checkins,
            listing,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::ListForResale {
            nonce: 0,
            ask_price: 1_500_000,
        }
        .data(),
    };
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[over_cap_listing],
        Some(&buyer.pubkey()),
        &[&buyer],
        blockhash,
    );
    assert!(context
        .banks_client
        .process_transaction(transaction)
        .await
        .is_err());
    assert!(context
        .banks_client
        .get_account(listing)
        .await
        .unwrap()
        .is_none());

    let list_instruction = Instruction {
        program_id: ENTRY_ID,
        accounts: accounts::ListForResale {
            seller: buyer.pubkey(),
            event,
            checkins,
            listing,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::ListForResale {
            nonce: 0,
            ask_price: 1_100_000,
        }
        .data(),
    };
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[list_instruction],
        Some(&buyer.pubkey()),
        &[&buyer],
        blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Listing creation is permissionless beyond the seller's signature, so
    // model corrupted/legacy state here to prove settlement rechecks the cap.
    let mut over_cap_listing_account = context
        .banks_client
        .get_account(listing)
        .await
        .unwrap()
        .unwrap();
    let mut over_cap_listing_state =
        entry::state::Listing::try_deserialize(&mut over_cap_listing_account.data.as_slice())
            .unwrap();
    over_cap_listing_state.ask_price = 1_500_000;
    let mut serialized_listing = Vec::new();
    over_cap_listing_state
        .try_serialize(&mut serialized_listing)
        .unwrap();
    over_cap_listing_account.data = serialized_listing;
    context.set_account(
        &listing,
        &solana_sdk::account::AccountSharedData::from(over_cap_listing_account.clone()),
    );

    let leaf_proof = |root: u8, flags: u8| entry::utils::bubblegum::TicketLeafProof {
        root: [root; 32],
        data_hash: [3; 32],
        creator_hash: [4; 32],
        asset_data_hash: None,
        flags: Some(flags),
        nonce: 0,
        index: 0,
    };
    let resale_proofs = entry::utils::bubblegum::TicketTransferProofs {
        thaw: leaf_proof(1, 2),
        transfer: leaf_proof(2, 0),
        refreeze: leaf_proof(3, 0),
    };
    let mut resale_accounts = accounts::BuyResale {
        buyer: resale_buyer.pubkey(),
        event,
        listing,
        seller: buyer.pubkey(),
        checkins,
        purchase_counter: Pubkey::find_program_address(
            &[b"purchases", event.as_ref(), resale_buyer.pubkey().as_ref()],
            &ENTRY_ID,
        )
        .0,
        config: config_key,
        payment_mint,
        buyer_usdc: resale_buyer_usdc,
        seller_usdc: buyer_usdc,
        organizer_usdc,
        fee_vault_usdc,
        tree_config: tree_config_key,
        merkle_tree: ticket_tree,
        collection,
        registry_tree,
        bubblegum_program: mpl_bubblegum::ID,
        compression_program: entry::constants::ACCOUNT_COMPRESSION_PROGRAM_ID,
        log_wrapper: entry::constants::NOOP_PROGRAM_ID,
        mpl_core_program: entry::constants::MPL_CORE_PROGRAM_ID,
        token_program: spl_token::ID,
        system_program: anchor_lang::system_program::ID,
    }
    .to_account_metas(None);
    resale_accounts.extend(
        proof_hashes
            .iter()
            .map(|hash| AccountMeta::new_readonly(Pubkey::new_from_array(*hash), false)),
    );
    let buy_resale = Instruction {
        program_id: ENTRY_ID,
        accounts: resale_accounts,
        data: instruction::BuyResale {
            args: entry::instructions::buy_resale::BuyResaleArgs {
                proofs: resale_proofs,
            },
        }
        .data(),
    };
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(1_000_000),
            buy_resale.clone(),
        ],
        Some(&resale_buyer.pubkey()),
        &[&resale_buyer],
        blockhash,
    );
    assert!(context
        .banks_client
        .process_transaction(transaction)
        .await
        .is_err());
    assert_eq!(
        token_amount(
            &context
                .banks_client
                .get_account(resale_buyer_usdc)
                .await
                .unwrap()
                .unwrap()
        ),
        2_000_000
    );
    assert!(context
        .banks_client
        .get_account(listing)
        .await
        .unwrap()
        .is_some());

    // Restore a valid 110% ask and verify the actual settlement path.
    let mut valid_listing_state = over_cap_listing_state;
    valid_listing_state.ask_price = 1_100_000;
    let mut serialized_listing = Vec::new();
    valid_listing_state
        .try_serialize(&mut serialized_listing)
        .unwrap();
    let mut valid_listing_account = over_cap_listing_account;
    valid_listing_account.data = serialized_listing;
    context.set_account(
        &listing,
        &solana_sdk::account::AccountSharedData::from(valid_listing_account),
    );

    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(1_000_000),
            buy_resale,
        ],
        Some(&resale_buyer.pubkey()),
        &[&resale_buyer],
        blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    assert_eq!(
        token_amount(
            &context
                .banks_client
                .get_account(resale_buyer_usdc)
                .await
                .unwrap()
                .unwrap()
        ),
        900_000
    );
    assert_eq!(
        token_amount(
            &context
                .banks_client
                .get_account(buyer_usdc)
                .await
                .unwrap()
                .unwrap()
        ),
        1_990_000
    );
    assert_eq!(
        token_amount(
            &context
                .banks_client
                .get_account(organizer_usdc)
                .await
                .unwrap()
                .unwrap()
        ),
        1_005_000
    );
    assert_eq!(
        token_amount(
            &context
                .banks_client
                .get_account(fee_vault_usdc)
                .await
                .unwrap()
                .unwrap()
        ),
        105_000
    );
    let event_account = context
        .banks_client
        .get_account(event)
        .await
        .unwrap()
        .unwrap();
    let event_state = Event::try_deserialize(&mut event_account.data.as_slice()).unwrap();
    assert_eq!(event_state.registry_seq, 2);
    let expected_resale_root = {
        let leaf = entry::utils::registry::owner_leaf(event, 0, resale_buyer.pubkey());
        let mut root = leaf;
        for sibling in &proof_hashes {
            root = solana_keccak_hasher::hashv(&[&root, sibling]).to_bytes();
        }
        root
    };
    assert_eq!(event_state.registry_root, expected_resale_root);
    assert!(context
        .banks_client
        .get_account(listing)
        .await
        .unwrap()
        .is_none());
    let tree = context
        .banks_client
        .get_account(registry_tree)
        .await
        .unwrap()
        .unwrap();
    let tree_state: &spl_account_compression::ConcurrentMerkleTree<14, 64> =
        bytemuck::from_bytes(&tree.data[56..]);
    assert_eq!(tree_state.get_root(), expected_resale_root);
    assert_eq!(tree_state.sequence_number, 2);
}

#[tokio::test]
#[ignore = "requires built ENTRY and SPL Account Compression SBF programs in BPF_OUT_DIR"]
async fn create_event_initializes_real_compression_tree_with_event_authority() {
    let bpf_dir = std::env::var("BPF_OUT_DIR").expect("set BPF_OUT_DIR to the two SBF artifacts");
    assert!(std::path::Path::new(&bpf_dir).join("entry.so").exists());
    assert!(std::path::Path::new(&bpf_dir)
        .join("spl_account_compression.so")
        .exists());

    let organizer = solana_sdk::signature::Keypair::new();
    let event_id = 89u64;
    let event = Pubkey::find_program_address(
        &[
            b"event",
            organizer.pubkey().as_ref(),
            &event_id.to_le_bytes(),
        ],
        &ENTRY_ID,
    )
    .0;
    let ticket_tree = Pubkey::new_unique();
    let registry_tree = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    let (tree_config_key, _) = TreeConfig::find_pda(&ticket_tree);
    let tree_config = TreeConfig {
        discriminator: [0; 8],
        tree_creator: organizer.pubkey(),
        tree_delegate: event,
        total_mint_capacity: 16_384,
        num_minted: 0,
        is_public: false,
        is_decompressible: DecompressibleState::Disabled,
        version: Version::V2,
    };
    let mut tree_config_data = Vec::new();
    tree_config.serialize(&mut tree_config_data).unwrap();
    let ticket_tree_space =
        56 + core::mem::size_of::<spl_account_compression::ConcurrentMerkleTree<14, 64>>();
    let mut ticket_tree_data = vec![0; ticket_tree_space];
    ticket_tree_data[0] = 1;
    ticket_tree_data[1] = 0;
    ticket_tree_data[2..6].copy_from_slice(&64u32.to_le_bytes());
    ticket_tree_data[6..10].copy_from_slice(&14u32.to_le_bytes());
    ticket_tree_data[10..42].copy_from_slice(tree_config_key.as_ref());

    let mut test = ProgramTest::new("entry", ENTRY_ID, None);
    test.add_account(
        organizer.pubkey(),
        Account {
            lamports: 10_000_000_000,
            data: Vec::new(),
            owner: anchor_lang::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_program(
        "spl_account_compression",
        entry::constants::ACCOUNT_COMPRESSION_PROGRAM_ID,
        None,
    );
    test.prefer_bpf(false);
    test.add_program(
        "spl_noop",
        entry::constants::NOOP_PROGRAM_ID,
        processor!(spl_noop::noop),
    );
    test.add_account(
        ticket_tree,
        Account {
            lamports: 10_000_000,
            data: ticket_tree_data,
            owner: entry::constants::ACCOUNT_COMPRESSION_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(
        tree_config_key,
        Account {
            lamports: 10_000_000,
            data: tree_config_data,
            owner: mpl_bubblegum::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    let registry_space =
        56 + core::mem::size_of::<spl_account_compression::ConcurrentMerkleTree<14, 64>>();
    test.add_account(
        registry_tree,
        Account {
            lamports: Rent::default().minimum_balance(registry_space) + 1,
            data: vec![0; registry_space],
            owner: entry::constants::ACCOUNT_COMPRESSION_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(
        collection,
        Account {
            lamports: 10_000_000,
            data: event_controlled_core_collection(event),
            owner: entry::constants::MPL_CORE_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    // These IDs are checked as fixed program accounts, but create_event does
    // not invoke Bubblegum or Core. The actual CMT CPI runs from its SBF file.
    for program_id in [mpl_bubblegum::ID, entry::constants::MPL_CORE_PROGRAM_ID] {
        test.add_account(
            program_id,
            Account {
                lamports: 1,
                data: Vec::new(),
                owner: Pubkey::default(),
                executable: false,
                rent_epoch: 0,
            },
        );
    }

    let context = test.start_with_context().await;
    let args = entry::instructions::create_event::CreateEventArgs {
        event_id,
        merkle_tree: ticket_tree,
        registry_tree,
        registry_depth: 14,
        collection,
        payment_mint: entry::constants::USDC_DEVNET_MINT,
        gate_authority: Pubkey::new_unique(),
        gatekeeper: Pubkey::new_unique(),
        price: 1_000_000,
        supply: 1,
        max_per_wallet: 1,
        resale_cap_bps: 11_000,
        royalty_bps: 500,
        ticket_name: "ENTRY".to_owned(),
        ticket_symbol: "ENT".to_owned(),
        ticket_uri: "https://example.invalid/tickets".to_owned(),
    };
    let checkins = Pubkey::find_program_address(&[b"checkins", event.as_ref()], &ENTRY_ID).0;
    let ix = Instruction {
        program_id: ENTRY_ID,
        accounts: accounts::CreateEvent {
            organizer: organizer.pubkey(),
            event,
            checkins,
            merkle_tree: ticket_tree,
            registry_tree,
            tree_config: tree_config_key,
            collection,
            bubblegum_program: mpl_bubblegum::ID,
            mpl_core_program: entry::constants::MPL_CORE_PROGRAM_ID,
            compression_program: entry::constants::ACCOUNT_COMPRESSION_PROGRAM_ID,
            log_wrapper: entry::constants::NOOP_PROGRAM_ID,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::CreateEvent { args }.data(),
    };
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(400_000),
            ix,
        ],
        Some(&organizer.pubkey()),
        &[&organizer],
        blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let event_account = context
        .banks_client
        .get_account(event)
        .await
        .unwrap()
        .unwrap();
    let event_state =
        entry::state::Event::try_deserialize(&mut event_account.data.as_slice()).unwrap();
    assert_eq!(event_state.registry_tree, registry_tree);
    assert_eq!(event_state.registry_depth, 14);
    assert_eq!(
        event_state.registry_root,
        entry::utils::registry::empty_root(14)
    );

    let tree = context
        .banks_client
        .get_account(registry_tree)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(tree.data[0], 1);
    assert_eq!(tree.data[1], 0);
    assert_eq!(u32::from_le_bytes(tree.data[2..6].try_into().unwrap()), 64);
    assert_eq!(u32::from_le_bytes(tree.data[6..10].try_into().unwrap()), 14);
    assert_eq!(&tree.data[10..42], event.as_ref());
}

#[tokio::test]
#[ignore = "requires the SPL Account Compression SBF program in BPF_OUT_DIR"]
async fn real_compression_program_replaces_a_leaf_with_a_valid_proof() {
    let bpf_dir = std::env::var("BPF_OUT_DIR").expect("set BPF_OUT_DIR to the SBF artifact");
    assert!(std::path::Path::new(&bpf_dir)
        .join("spl_account_compression.so")
        .exists());

    let authority = Keypair::new();
    let tree = Pubkey::new_unique();
    let depth = 14u32;
    let tree_size =
        56 + core::mem::size_of::<spl_account_compression::ConcurrentMerkleTree<14, 64>>();
    let mut test = ProgramTest::default();
    test.add_account(
        authority.pubkey(),
        Account {
            lamports: 10_000_000_000,
            data: Vec::new(),
            owner: anchor_lang::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_program(
        "spl_account_compression",
        entry::constants::ACCOUNT_COMPRESSION_PROGRAM_ID,
        None,
    );
    test.prefer_bpf(false);
    test.add_program(
        "spl_noop",
        entry::constants::NOOP_PROGRAM_ID,
        processor!(spl_noop::noop),
    );
    test.add_account(
        tree,
        Account {
            lamports: Rent::default().minimum_balance(tree_size) + 1,
            data: vec![0; tree_size],
            owner: entry::constants::ACCOUNT_COMPRESSION_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    let context = test.start_with_context().await;
    let initialize = Instruction {
        program_id: entry::constants::ACCOUNT_COMPRESSION_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(tree, false),
            AccountMeta::new_readonly(authority.pubkey(), true),
            AccountMeta::new_readonly(entry::constants::NOOP_PROGRAM_ID, false),
        ],
        data: compression_instruction_data(
            b"global:init_empty_merkle_tree",
            &spl_account_compression::instruction::InitEmptyMerkleTree {
                max_depth: depth,
                max_buffer_size: 64,
            },
        ),
    };
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(400_000),
            initialize,
        ],
        Some(&authority.pubkey()),
        &[&authority],
        blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let mut proof = Vec::with_capacity(depth as usize);
    let mut empty_node = [0u8; 32];
    for _ in 0..depth {
        proof.push(empty_node);
        empty_node = solana_keccak_hasher::hashv(&[&empty_node, &empty_node]).to_bytes();
    }
    let old_root = empty_node;
    let new_leaf = [0x5a; 32];
    let proof_accounts: Vec<_> = proof
        .iter()
        .map(|node| AccountMeta::new_readonly(Pubkey::new_from_array(*node), false))
        .collect();
    let mut replace_accounts = vec![
        AccountMeta::new(tree, false),
        AccountMeta::new_readonly(authority.pubkey(), true),
        AccountMeta::new_readonly(entry::constants::NOOP_PROGRAM_ID, false),
    ];
    replace_accounts.extend(proof_accounts);
    let replace = Instruction {
        program_id: entry::constants::ACCOUNT_COMPRESSION_PROGRAM_ID,
        accounts: replace_accounts,
        data: compression_instruction_data(
            b"global:replace_leaf",
            &spl_account_compression::instruction::ReplaceLeaf {
                root: old_root,
                previous_leaf: [0; 32],
                new_leaf,
                index: 0,
            },
        ),
    };
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(400_000),
            replace,
        ],
        Some(&authority.pubkey()),
        &[&authority],
        blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let tree_account = context
        .banks_client
        .get_account(tree)
        .await
        .unwrap()
        .unwrap();
    let tree_state: &spl_account_compression::ConcurrentMerkleTree<14, 64> =
        bytemuck::from_bytes(&tree_account.data[56..]);
    let mut expected_root = new_leaf;
    for sibling in &proof {
        expected_root = solana_keccak_hasher::hashv(&[&expected_root, sibling]).to_bytes();
    }
    assert_eq!(tree_state.get_root(), expected_root);
    assert_eq!(tree_state.sequence_number, 1);

    let stale_proof_accounts: Vec<_> = proof
        .iter()
        .map(|node| AccountMeta::new_readonly(Pubkey::new_from_array(*node), false))
        .collect();
    let mut stale_accounts = vec![
        AccountMeta::new(tree, false),
        AccountMeta::new_readonly(authority.pubkey(), true),
        AccountMeta::new_readonly(entry::constants::NOOP_PROGRAM_ID, false),
    ];
    stale_accounts.extend(stale_proof_accounts);
    let stale_replace = Instruction {
        program_id: entry::constants::ACCOUNT_COMPRESSION_PROGRAM_ID,
        accounts: stale_accounts,
        data: compression_instruction_data(
            b"global:replace_leaf",
            &spl_account_compression::instruction::ReplaceLeaf {
                root: old_root,
                previous_leaf: [0; 32],
                new_leaf: [0x33; 32],
                index: 0,
            },
        ),
    };
    let blockhash = context.banks_client.get_latest_blockhash().await.unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(400_000),
            stale_replace,
        ],
        Some(&authority.pubkey()),
        &[&authority],
        blockhash,
    );
    assert!(context
        .banks_client
        .process_transaction(transaction)
        .await
        .is_err());

    let tree_account = context
        .banks_client
        .get_account(tree)
        .await
        .unwrap()
        .unwrap();
    let tree_state: &spl_account_compression::ConcurrentMerkleTree<14, 64> =
        bytemuck::from_bytes(&tree_account.data[56..]);
    assert_eq!(tree_state.get_root(), expected_root);
    assert_eq!(tree_state.sequence_number, 1);
}
