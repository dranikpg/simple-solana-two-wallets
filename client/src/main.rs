use console::style;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

mod accounts;
mod utils;

fn encode_instruction_data(token_type: u8, amount: u64) -> [u8; 9] {
    let mut out = [0u8; 9];
    out[0] = token_type;
    out[1..].copy_from_slice(&amount.to_le_bytes());
    out
}

fn create_wallet(client: &RpcClient, prog_id: &Keypair, payer_id: &Keypair) -> Keypair {
    println!("Creating user wallet");
    let lamports = utils::read_u64("Lamports");
    let wallet = Keypair::new();
    accounts::create(&client, &wallet, &prog_id, &payer_id, lamports)
        .expect("Failed to create wallet account");
    wallet
}

fn create_pda(client: &RpcClient, prog_id: &Keypair, payer_id: &Keypair, seed: &str) -> Pubkey {
    let pda = Pubkey::create_with_seed(&prog_id.pubkey(), seed, &prog_id.pubkey()).unwrap();
    match client.get_account(&pda) {
        Err(_) => {
            println!("Creating pda wallet");
            let lamports = utils::read_u64("Lamports");
            accounts::create_with_seed(&client, &prog_id, &payer_id, &pda, seed, lamports)
                .expect("Failed to create pda account");
        }
        Ok(account) => {
            println!(
                "{} pda account with balance {}",
                style("Found").green(),
                account.lamports
            );
        }
    }
    pda
}

fn check_account(client: &RpcClient, pubkey: &Pubkey) -> Option<Account> {
    let account = client.get_account(pubkey);
    if let Ok(ref account) = &account {
        utils::print_account_info(pubkey, account);
    } else {
        println!("{} account for {}", style("Missing").red(), pubkey);
    }
    account.ok()
}

fn main() {
    let client = RpcClient::new("http://localhost:8899".to_string());
    let prog_id = utils::read_key("prog.id");
    let payer_id = utils::read_key("payer.id");

    check_account(&client, &prog_id.pubkey());
    check_account(&client, &payer_id.pubkey());

    let create_wallet = || create_wallet(&client, &prog_id, &payer_id);
    let create_pda = |seed| create_pda(&client, &prog_id, &payer_id, seed);

    println!("{} setup", style("Running").yellow());
    let (debit_x, debit_y) = (create_pda("t11_x"), create_pda("t11_y"));
    let (wallet_x, wallet_y) = (create_wallet(), create_wallet());

    let print_balance = || utils::print_balance(&client,
        &[
            (&debit_x, "pda x"),
            (&debit_y, "pda y"),
            (&wallet_x.pubkey(), "wallet x"),
            (&wallet_y.pubkey(), "wallet y"),
        ],
    );
    

    loop {
        print_balance();
        utils::print_invariant(&client, &debit_x, &debit_y);

        let token_type = dialoguer::Select::new()
            .with_prompt("select token to transfer")
            .items(&["token x", "token y"])
            .default(0)
            .interact()
            .expect("Failed to read token");

        let amount = utils::read_u64("select amount");

        let data = &encode_instruction_data(token_type as u8 + 1, amount);
        println!("Encoded transaction data: {:?}", data);

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bincode(
                prog_id.pubkey().clone(),
                data,
                vec![
                    AccountMeta::new(debit_x, false),
                    AccountMeta::new(debit_y, false),
                    AccountMeta::new(wallet_x.pubkey(), false),
                    AccountMeta::new(wallet_y.pubkey(), false),
                ],
            )],
            Some(&payer_id.pubkey()),
        );
        transaction
            .try_sign(&[&payer_id], client.get_recent_blockhash()
                .expect("Failed to get blockhash").0)
            .expect("Failed to sign");
        client
            .send_and_confirm_transaction_with_spinner(&transaction)
            .expect("transaction failed");
    }
}
