use std::{io::prelude::*, sync::Arc};

use aries_vcx_agent::{
    aries_vcx::aries_vcx_wallet::wallet::askar::AskarWallet, build_askar_wallet,
    Agent as AriesAgent, WalletInitConfig,
};
use rand::{rng, Rng};
use reqwest::Url;

#[derive(Debug, Deserialize)]
struct SeedResponse {
    seed: String,
}

struct WriterSeed {
    pub seed: String,
    pub is_trustee: bool,
}

/// If a ledger URL is available, then use it to register a new endorser, then return this
/// endorser's DID seed.
/// If not available, then default to using the local ledger Trustee seed.
async fn get_writer_seed() -> WriterSeed {
    if let Ok(ledger_url) = std::env::var("LEDGER_URL") {
        let url = format!("{}/register", ledger_url);
        let mut rng = rng();
        let client = reqwest::Client::new();
        let body = json!({
            "role": "TRUST_ANCHOR",
            "seed": format!("my_seed_000000000000000000{}", rng.random_range(100000..1000000))
        })
        .to_string();
        let seed = client
            .post(&url)
            .body(body)
            .send()
            .await
            .expect("Failed to send message")
            .json::<SeedResponse>()
            .await
            .expect("Failed to deserialize response")
            .seed;
        WriterSeed {
            seed,
            is_trustee: false,
        }
    } else {
        WriterSeed {
            seed: "000000000000000000000000Trustee1".to_string(),
            is_trustee: true,
        }
    }
}

async fn download_genesis_file() -> std::result::Result<String, String> {
    match std::env::var("GENESIS_FILE").ok() {
        Some(genesis_file) => {
            if !std::path::Path::new(&genesis_file).exists() {
                Err(format!("The file {} does not exist", genesis_file))
            } else {
                info!("Using genesis file {}", genesis_file);
                Ok(genesis_file)
            }
        }
        None => match std::env::var("LEDGER_URL").ok() {
            Some(ledger_url) => {
                info!("Downloading genesis file from {}", ledger_url);
                let genesis_url = format!("{}/genesis", ledger_url);
                let body = reqwest::get(&genesis_url)
                    .await
                    .expect("Failed to get genesis file from ledger")
                    .text()
                    .await
                    .expect("Failed to get the response text");
                let path = std::env::current_dir()
                    .expect("Failed to obtain the current directory path")
                    .join("resource")
                    .join("genesis_file.txn");
                info!("Storing genesis file to {:?}", path);
                let mut f = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path.clone())
                    .expect("Unable to open file");
                f.write_all(body.as_bytes()).expect("Unable to write data");
                debug!("Genesis file downloaded and saved to {:?}", path);
                path.to_str()
                    .map(|s| s.to_string())
                    .ok_or("Failed to convert genesis file path to string".to_string())
            }
            None => std::env::current_dir()
                .expect("Failed to obtain the current directory path")
                .join("resource")
                .join("indypool.txn")
                .to_str()
                .map(|s| s.to_string())
                .ok_or("Failed to convert genesis file path to string".to_string()),
        },
    }
}

pub async fn initialize(port: u32) -> AriesAgent<AskarWallet> {
    let register_nym_res = get_writer_seed().await;
    let genesis_path = download_genesis_file()
        .await
        .expect("Failed to download the genesis file");
    let dockerhost = std::env::var("DOCKERHOST").unwrap_or("localhost".to_string());
    let service_endpoint = Url::parse(&format!("http://{}:{}/didcomm", dockerhost, port)).unwrap();

    let wallet_config = WalletInitConfig {
        wallet_name: format!("rust_agent_{}", uuid::Uuid::new_v4()),
        wallet_key: "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY".to_string(),
        wallet_kdf: "RAW".to_string(),
    };
    let (wallet, issuer_config) = build_askar_wallet(wallet_config, register_nym_res.seed).await;
    let wallet = Arc::new(wallet);

    let issuer_did = AriesAgent::setup_ledger(
        genesis_path.clone(),
        wallet.clone(),
        service_endpoint.clone(),
        issuer_config.institution_did.parse().unwrap(),
        register_nym_res.is_trustee,
    )
    .await
    .unwrap();

    AriesAgent::initialize(
        genesis_path,
        wallet.clone(),
        service_endpoint.clone(),
        issuer_did,
    )
    .await
    .unwrap()
}
