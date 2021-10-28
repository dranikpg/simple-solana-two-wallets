use {
    solana_escrow::process_instruction,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
    solana_program_test::*,
    solana_sdk::{
        account::Account,
        signature::{Keypair, Signer},
        transaction::Transaction,
    },
    std::str::FromStr,
};

fn create_pda_account(
    program_id: &Pubkey,
    test: &mut ProgramTest,
    lamports: u64,
    seed: &[u8],
) -> Pubkey {
    let pubkey =
        Pubkey::create_program_address(&[seed], program_id).expect("Failed to create pda account");
    test.add_account(
        pubkey,
        Account {
            lamports,
            owner: *program_id,
            ..Account::default()
        },
    );
    pubkey
}

fn create_user_wallet(program_id: &Pubkey, test: &mut ProgramTest, lamports: u64) -> Keypair {
    let keypair = Keypair::new();
    test.add_account(
        keypair.pubkey(),
        Account {
            lamports,
            owner: *program_id,
            ..Account::default()
        },
    );
    keypair
}

fn encode_instruction_data(token_type: u8, amount: u64) -> [u8; 9] {
    let mut out = [0u8; 9];
    out[0] = token_type;
    out[1..].copy_from_slice(&amount.to_le_bytes());
    out
}

#[tokio::test]
async fn test_transfer() {
    let program_id = Pubkey::from_str("TransferLamports111111111111111111111111111").unwrap();
    let mut program_test =
        ProgramTest::new("test_transfer", program_id, processor!(process_instruction));

    let debit_x = create_pda_account(&program_id, &mut program_test, 6, b"pda_x");
    let debit_y = create_pda_account(&program_id, &mut program_test, 10, b"pda_y");
    let wallet_x = create_user_wallet(&program_id, &mut program_test, 6);
    let wallet_y = create_user_wallet(&program_id, &mut program_test, 0);

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &encode_instruction_data(1, 6),
            vec![
                AccountMeta::new(debit_x, false),
                AccountMeta::new(debit_y, false),
                AccountMeta::new(wallet_x.pubkey(), false),
                AccountMeta::new(wallet_y.pubkey(), false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    assert_eq!(banks_client.get_balance(debit_x).await.unwrap(), 12);
    assert_eq!(banks_client.get_balance(debit_y).await.unwrap(), 5);

    assert_eq!(
        banks_client.get_balance(wallet_x.pubkey()).await.unwrap(),
        0
    );
    assert_eq!(
        banks_client.get_balance(wallet_y.pubkey()).await.unwrap(),
        5
    );
}
