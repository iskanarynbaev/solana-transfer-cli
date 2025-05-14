use futures::future::join_all;
use serde::Deserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Signature, read_keypair_file},
    system_transaction,
};
use std::{fs, time::Instant};

#[derive(Debug, Deserialize)]
struct TransferConfig {
    rpc_url: String,
    transfers: Vec<Transfer>,
}

#[derive(Debug, Deserialize)]
struct Transfer {
    from_keypair: String,
    to: String,
    amount: f64,
}

async fn send_transaction(rpc_url: &str, transfer: &Transfer) -> (Result<Signature, String>, u128) {
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let from = read_keypair_file(&transfer.from_keypair)
        .map_err(|e| e.to_string())
        .unwrap();
    let to = transfer.to.parse().unwrap();
    let lamports = solana_sdk::native_token::sol_to_lamports(transfer.amount);

    let blockhash = client.get_latest_blockhash().unwrap();
    let tx = system_transaction::transfer(&from, &to, lamports, blockhash);

    let start = Instant::now();
    let result = client.send_and_confirm_transaction(&tx);
    let duration = start.elapsed().as_millis();

    (result.map_err(|e| e.to_string()), duration)
}

#[tokio::main]
async fn main() {
    let config_str = fs::read_to_string("config.yaml").expect("Failed to read config.yaml");
    let config: TransferConfig = serde_yaml::from_str(&config_str).expect("Failed to parse config");

    let tasks = config.transfers.iter().map(|t| {
        let rpc_url = config.rpc_url.clone();
        async move {
            let (res, time) = send_transaction(&rpc_url, t).await;
            match res {
                Ok(sig) => println!("✅ Tx sent: {} ({} ms)", sig, time),
                Err(e) => println!("❌ Error: {} ({} ms)", e, time),
            }
        }
    });

    join_all(tasks).await;
}
