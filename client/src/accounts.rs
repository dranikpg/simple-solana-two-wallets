use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    message::Message, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction,
};

pub fn create(
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

pub fn create_with_seed(
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
    let transaction = Transaction::new(
        &[payer_id, prog_id],
        message,
        client.get_recent_blockhash()?.0,
    );
    client.send_and_confirm_transaction_with_spinner(&transaction)?;
    Ok(())
}
