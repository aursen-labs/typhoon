use {
    base64::{prelude::BASE64_STANDARD, Engine},
    events::Counter,
    litesvm::LiteSVM,
    solana_address::Address,
    solana_keypair::Keypair,
    solana_native_token::LAMPORTS_PER_SOL,
    solana_signer::Signer,
    solana_transaction::Transaction,
    std::path::PathBuf,
    typhoon_instruction_builder::generate_instructions_client,
};

const ID: Address = Address::from_str_const("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// `Incremented` is the first variant of the `CounterEvent` enum, so its
// discriminator is the index `0` encoded as a single `u8`.
const INCREMENTED_DISCRIMINATOR: u8 = 0;

fn read_program() -> Vec<u8> {
    let mut so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    so_path.push("target/deploy/events.so");

    std::fs::read(so_path).unwrap()
}

generate_instructions_client!(events);

#[test]
fn emits_event_in_logs() {
    let mut svm = LiteSVM::new();
    let admin_kp = Keypair::new();
    let admin_pk = admin_kp.pubkey();

    svm.airdrop(&admin_pk, 10 * LAMPORTS_PER_SOL).unwrap();
    svm.add_program(ID, &read_program()).unwrap();

    // Create the counter.
    let counter_kp = Keypair::new();
    let counter_pk = counter_kp.pubkey();
    let ix = InitializeInstruction {
        init: InitContext {
            payer: admin_pk,
            counter: counter_pk,
            system: solana_system_interface::program::ID,
        },
    }
    .into_instruction();
    let hash = svm.latest_blockhash();
    let tx =
        Transaction::new_signed_with_payer(&[ix], Some(&admin_pk), &[&admin_kp, &counter_kp], hash);
    svm.send_transaction(tx).unwrap();

    // Increment the counter — this emits the `Incremented` event.
    let ix = IncrementInstruction {
        ctx: CounterMutContext {
            counter: counter_pk,
        },
    }
    .into_instruction();
    let hash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&admin_pk), &[&admin_kp], hash);
    let meta = svm.send_transaction(tx).unwrap();

    // The account state was updated.
    let raw_account = svm.get_account(&counter_pk).unwrap();
    let counter_account: &Counter = bytemuck::from_bytes(&raw_account.data[8..]);
    assert_eq!(counter_account.count, 1);

    // The event shows up as a single `Program data:` log field: the variant
    // discriminator byte followed by the packed variant fields.
    let payload = meta
        .logs
        .iter()
        .find_map(|line| line.strip_prefix("Program data: "))
        .map(|data| BASE64_STANDARD.decode(data).unwrap())
        .expect("expected a `Program data:` log line for the emitted event");

    assert_eq!(payload[0], INCREMENTED_DISCRIMINATOR);

    let count = u64::from_le_bytes(payload[1..9].try_into().unwrap());
    assert_eq!(count, 1);
}

#[test]
fn event_is_in_the_idl() {
    let idl_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/idl/events.json");
    let idl = std::fs::read_to_string(idl_path).unwrap();

    assert!(
        idl.contains("\"eventNode\"") && idl.contains("\"incremented\""),
        "the generated IDL should contain the `incremented` event"
    );
}
