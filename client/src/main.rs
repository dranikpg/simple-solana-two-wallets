use console::style;
use dialoguer::Input;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair},
    signer::Signer,
    transaction::Transaction,
};

fn read_u64(msg: &'static str) -> u64 {
    loop {
        let raw: String = Input::new()
            .with_prompt(msg)
            .validate_with(|input: &str| -> Result<(), &str> {
                input
                    .parse::<u64>()
                    .map(|_| ())
                    .map_err(|_| "Must be positive integer")
            })
            .interact()
            .unwrap();
        break raw.parse::<u64>().unwrap_or(0);
    }
}

fn read_key(path: &str) -> Keypair {
    match read_keypair_file(path) {
        Ok(kp) => {
            println!("{} key {}", style("Found").green(), path);
            kp
        }
        Err(err) => {
            println!(
                "{} to parse private key at ./{}",
                style("Failed").red(),
                style(path).bold()
            );
            println!("Reason: {:?}", err);
            std::process::exit(-1);
        }
    }
}

fn encode_instruction_data(token_type: u8, amount: u64) -> [u8; 9] {
    let mut out = [0u8; 9];
    out[0] = token_type;
    out[1..].copy_from_slice(&amount.to_le_bytes());
    out
}

fn create_account(
    client: &RpcClient,
    keypair: &Keypair,
    program_id: &Keypair,
    owner_id: &Keypair,
    lamports: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let instruction = solana_sdk::system_instruction::create_account(
        &owner_id.pubkey(),
        &keypair.pubkey(),
        lamports,
        0,
        &program_id.pubkey(),
    );
    let message = Message::new(&[instruction], Some(&owner_id.pubkey()));
    let transaction = Transaction::new(
        &[owner_id, &keypair],
        message,
        client.get_recent_blockhash()?.0,
    );
    client.send_and_confirm_transaction_with_spinner(&transaction)?;
    Ok(())
}

fn create_account_with_seed(
    client: &RpcClient,
    prog_id: &Keypair,
    payer_id: &Keypair,
    pubkey: &Pubkey,
    seed: &str,
    lamports: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let instruction = solana_sdk::system_instruction::create_account_with_seed(
        &payer_id.pubkey(),
        &pubkey,
        &prog_id.pubkey(),
        seed,
        lamports,
        0,
        &prog_id.pubkey(),
    );
    let message = Message::new(&[instruction], Some(&payer_id.pubkey()));
    let transaction = Transaction::new(&[payer_id, prog_id], message, client.get_recent_blockhash()?.0);
    client.send_and_confirm_transaction_with_spinner(&transaction)?;
    Ok(())
}

fn print_balance(client: &RpcClient, items: &[(&Pubkey, &'static str)]) {
    let width = items.iter().map(|(_, name)| name.len()).max().unwrap_or(0);
    println!("==== {} =====", style("Balance").bold());
    for (key, name) in items {
        let balance = client.get_balance(&key).expect("Failed to get balance");
        println!("{:width$}  {}", name, style(balance).green(), width = width);
    }
    println!("==================")
}

fn print_invariant(client: &RpcClient, debit_x: &Pubkey, debit_y: &Pubkey) {
    let balance_x = client
        .get_balance(debit_x)
        .expect("Failed to fetch balance");
    let balance_y = client
        .get_balance(debit_y)
        .expect("Failed to fetch balance");
    let product = balance_x as f64 * 1.0 * balance_y as f64 / 1e9;
    println!("==== {} ===", style("Invariant").bold());
    println!("Product: {} * 10^9", product);
    println!("==================")
}

fn print_account_info(client: &RpcClient, pubkey: &Pubkey) {
    let account = client.get_account(pubkey);
    println!("==== {} =====", style("Account").bold());
    println!("Pubkey: {}", pubkey);
    if let Ok(account) = account {
        println!("{:#?}", account);
    } else {
        println!("{} account", style("Missing").red());
    }
    println!("==================")
}

fn main() {
    let client = RpcClient::new("http://localhost:8899".to_string());
    let prog_id = read_key("prog.id");
    let payer_id = read_key("payer.id");

    print_account_info(&client, &prog_id.pubkey());
    print_account_info(&client, &payer_id.pubkey());

    let create_wallet = || {
        println!("Creating user wallet");
        let lamports = read_u64("Lamports");
        let wallet = Keypair::new();
        create_account(&client, &wallet, &prog_id, &payer_id, lamports)
            .expect("Failed to create wallet account");
        wallet
    };

    let create_pda = |seed: &str| {
        let debit = Pubkey::create_with_seed(&prog_id.pubkey(), seed, &prog_id.pubkey()).unwrap();
        match client.get_account(&debit) {
            Err(_) => {
                println!("Creating pda wallet");
                let lamports = read_u64("Lamports");
                create_account_with_seed(&client, &prog_id, &payer_id, &debit, seed, lamports)
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
        debit
    };

    println!("{} setup", style("Running").yellow());
    let (debit_x, debit_y) = (create_pda("t11_x"), create_pda("t11_y"));
    let (wallet_x, wallet_y) = (create_wallet(), create_wallet());

    let print_balance = || {
        print_balance(
            &client,
            &[
                (&debit_x, "pda x"),
                (&debit_y, "pda y"),
                (&wallet_x.pubkey(), "wallet x"),
                (&wallet_y.pubkey(), "wallet y"),
            ],
        );
    };

    loop {
        print_balance();
        print_invariant(&client, &debit_x, &debit_y);

        let token_type = dialoguer::Select::new()
            .with_prompt("select token to transfer")
            .items(&["token x", "token y"])
            .default(0)
            .interact()
            .unwrap();
        let amount = read_u64("select amount");

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
            .try_sign(&[&payer_id], client.get_recent_blockhash().expect("Failed to get blockhash").0 )
            .expect("Failed to sign");
        client
            .send_and_confirm_transaction_with_spinner(&transaction)
            .expect("transaction failed");
    }
}
