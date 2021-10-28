use console::style;
use dialoguer::Input;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{account::Account, pubkey::Pubkey, signature::{Keypair, read_keypair_file}};

pub fn read_u64(msg: &'static str) -> u64 {
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

pub fn read_key(path: &str) -> Keypair {
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

pub fn print_balance(client: &RpcClient, items: &[(&Pubkey, &'static str)]) {
    let width = items.iter().map(|(_, name)| name.len()).max().unwrap_or(0);
    println!("==== {} =====", style("Balance").bold());
    for (key, name) in items {
        let balance = client.get_balance(&key).expect("Failed to get balance");
        println!("{:width$}  {}", name, style(balance).green(), width = width);
    }
    println!("==================")
}

pub fn print_invariant(client: &RpcClient, debit_x: &Pubkey, debit_y: &Pubkey) {
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

pub fn print_account_info(pubkey: &Pubkey, account: &Account) {
    println!("==== {} =====", style("Account").bold());
    println!("Pubkey: {}", pubkey);
    println!("{:#?}", account);
    println!("==================")
}
