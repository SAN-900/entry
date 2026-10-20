use anchor_lang::{
    AccountDeserialize, AccountSerialize, Discriminator, InstructionData, ToAccountMetas,
};
use entry::{
    accounts, instruction,
    state::{CheckIns, Event},
    ID as ENTRY_ID,
};
use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

const EVENT_SEED: &[u8] = b"event";
const CHECKIN_SEED: &[u8] = b"checkins";
fn event_account(
    organizer: Pubkey,
    event_id: u64,
    gate_authority: Pubkey,
    supply: u32,
    sold: u32,
) -> (Pubkey, Account) {
    let (key, bump) = Pubkey::find_program_address(
        &[EVENT_SEED, organizer.as_ref(), &event_id.to_le_bytes()],
        &ENTRY_ID,
    );
    let event = Event {
        organizer,
        event_id,
        gate_authority,
        gatekeeper: Pubkey::new_unique(),
        payment_mint: Pubkey::new_unique(),
        merkle_tree: Pubkey::new_unique(),
        registry_tree: Pubkey::new_unique(),
        registry_root: [0; 32],
        registry_depth: 3,
        registry_seq: 0,
        collection: Pubkey::new_unique(),
        price: 1_000_000,
        supply,
        sold,
        max_per_wallet: 2,
        resale_cap_bps: 11_000,
        royalty_bps: 500,
        ticket_name: "ENTRY".to_owned(),
        ticket_symbol: "ENT".to_owned(),
        ticket_uri: "https://example.invalid/ticket".to_owned(),
        active: true,
        bump,
    };
    let mut data = Vec::new();
    event.try_serialize(&mut data).unwrap();
    (
        key,
        Account {
            lamports: 10_000_000,
            data,
            owner: ENTRY_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

fn checkins_account(event: Pubkey, supply: u32) -> (Pubkey, Account) {
    let (key, _) = Pubkey::find_program_address(&[CHECKIN_SEED, event.as_ref()], &ENTRY_ID);
    let mut data = vec![0; CheckIns::account_space(supply)];
    data[..8].copy_from_slice(&CheckIns::DISCRIMINATOR);
    data[8..40].copy_from_slice(event.as_ref());
    data[40..44].copy_from_slice(&supply.to_le_bytes());
    (
        key,
        Account {
            lamports: 10_000_000,
            data,
            owner: ENTRY_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

fn program_test() -> ProgramTest {
    ProgramTest::new("entry", ENTRY_ID, processor!(entrypoint))
}

fn entrypoint<'info>(
    program_id: &Pubkey,
    accounts: &[anchor_lang::solana_program::account_info::AccountInfo<'info>],
    instruction_data: &[u8],
) -> anchor_lang::solana_program::entrypoint::ProgramResult {
    // Anchor's generated entrypoint ties the account-slice borrow to each
    // AccountInfo lifetime, while ProgramTest's processor type leaves them
    // independent. ProgramTest invokes this synchronously, and Anchor does
    // not retain account references after entry returns.
    let accounts: &'info [_] = unsafe { core::mem::transmute(accounts) };
    entry::entry(program_id, accounts, instruction_data)
}

async fn process_instruction(
    context: &mut ProgramTestContext,
    instruction: Instruction,
    additional_signers: &[&Keypair],
) -> Result<(), solana_program_test::BanksClientError> {
    let blockhash = context.banks_client.get_latest_blockhash().await?;
    let mut signers: Vec<&dyn Signer> = vec![&context.payer];
    signers.extend(
        additional_signers
            .iter()
            .map(|signer| *signer as &dyn Signer),
    );
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &signers,
        blockhash,
    );
    context.banks_client.process_transaction(transaction).await
}

#[tokio::test]
async fn organizer_can_pause_and_resume_event() {
    let organizer = Keypair::new();
    let (event_key, event_account) =
        event_account(organizer.pubkey(), 7, Pubkey::new_unique(), 3, 0);
    let mut test = program_test();
    test.add_account(event_key, event_account);
    let mut context = test.start_with_context().await;

    for active in [false, true] {
        let ix = Instruction {
            program_id: ENTRY_ID,
            accounts: accounts::SetActive {
                organizer: organizer.pubkey(),
                event: event_key,
            }
            .to_account_metas(None),
            data: instruction::SetActive { active }.data(),
        };
        process_instruction(&mut context, ix, &[&organizer])
            .await
            .unwrap();
        let account = context
            .banks_client
            .get_account(event_key)
            .await
            .unwrap()
            .unwrap();
        let updated = Event::try_deserialize(&mut account.data.as_slice()).unwrap();
        assert_eq!(updated.active, active);
    }
}

#[tokio::test]
async fn gate_checkin_batch_sets_bits_once_and_rejects_unsold_nonce() {
    let organizer = Pubkey::new_unique();
    let gate = Keypair::new();
    let (event_key, mut event_account) = event_account(organizer, 19, gate.pubkey(), 3, 2);
    let owner = Pubkey::new_unique();
    let proof_keys: Vec<_> = (1..=14u8)
        .map(|byte| Pubkey::new_from_array([byte; 32]))
        .collect();
    let owner_leaf = entry::utils::registry::owner_leaf(event_key, 0, owner);
    let mut registry_root = owner_leaf;
    for (level, sibling) in proof_keys.iter().enumerate() {
        registry_root = if ((0u64 >> level) & 1) == 0 {
            solana_keccak_hasher::hashv(&[&registry_root, sibling.as_ref()]).to_bytes()
        } else {
            solana_keccak_hasher::hashv(&[sibling.as_ref(), &registry_root]).to_bytes()
        };
    }
    let mut event_state = Event::try_deserialize(&mut event_account.data.as_slice()).unwrap();
    event_state.registry_root = registry_root;
    event_state.registry_depth = 14;
    let mut event_data = Vec::new();
    event_state.try_serialize(&mut event_data).unwrap();
    event_account.data = event_data;
    let (checkins_key, checkins_account) = checkins_account(event_key, 3);
    let mut test = program_test();
    test.add_account(event_key, event_account);
    test.add_account(checkins_key, checkins_account);
    let mut context = test.start_with_context().await;

    let proof_metas: Vec<_> = proof_keys
        .iter()
        .map(|key| solana_sdk::instruction::AccountMeta::new_readonly(*key, false))
        .collect();
    let make_ix = |tickets: Vec<entry::instructions::check_in::CheckInTicket>| {
        let mut accounts = accounts::CheckInBatch {
            gate: gate.pubkey(),
            event: event_key,
            checkins: checkins_key,
        }
        .to_account_metas(None);
        for _ in 0..tickets.len() {
            accounts.extend(proof_metas.clone());
        }
        Instruction {
            program_id: ENTRY_ID,
            accounts,
            data: instruction::CheckInBatch { tickets }.data(),
        }
    };
    let duplicate_scans = vec![
        entry::instructions::check_in::CheckInTicket { nonce: 0, owner },
        entry::instructions::check_in::CheckInTicket { nonce: 0, owner },
    ];
    process_instruction(&mut context, make_ix(duplicate_scans), &[&gate])
        .await
        .unwrap();

    let account = context
        .banks_client
        .get_account(checkins_key)
        .await
        .unwrap()
        .unwrap();
    let bitmap = &account.data[CheckIns::HEADER_SIZE..];
    assert_eq!(bitmap[0] & 0b01, 0b01);

    assert!(process_instruction(
        &mut context,
        make_ix(vec![entry::instructions::check_in::CheckInTicket {
            nonce: 0,
            owner: Pubkey::new_unique(),
        }]),
        &[&gate]
    )
    .await
    .is_err());

    assert!(process_instruction(
        &mut context,
        make_ix(vec![entry::instructions::check_in::CheckInTicket {
            nonce: 2,
            owner
        }]),
        &[&gate]
    )
    .await
    .is_err());
}
