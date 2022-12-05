use std::io::Write;

use anyhow::{anyhow, Context};
use rand::{thread_rng, Rng};
use serde::Deserialize;

use super::init_config::InitConfig;

#[derive(Deserialize)]
struct SeedResponse {
    seed: String,
}

pub async fn assure_trustee_seed(init_config: &InitConfig) -> anyhow::Result<String> {
    if let Some(seed) = init_config.trustee_seed() {
        return Ok(seed.to_string());
    };
    if let Some(ledger_url) = init_config.ledger_url() {
        let url = format!("{}/register", ledger_url);
        let client = reqwest::Client::new();
        let body = json!({
            "role": "TRUST_ANCHOR",
            "seed": format!("my_seed_000000000000000000{}", thread_rng().gen_range(100000..1000000))
        })
        .to_string();
        Ok(client
            .post(&url)
            .body(body)
            .send()
            .await
            .context("Failed to send message")?
            .json::<SeedResponse>()
            .await
            .context("Failed to deserialize response")?
            .seed)
    } else {
        Err(anyhow!("No trustee seed found in config and no ledger url provided"))
    }
}

pub async fn assure_genesis_file(config: &InitConfig) -> anyhow::Result<String> {
    match config.genesis_file() {
        Some(genesis_file) => {
            if !std::path::Path::new(&genesis_file).exists() {
                Err(anyhow!("The file {} does not exist", genesis_file))
            } else {
                Ok(genesis_file.to_string())
            }
        }
        None => match config.ledger_url() {
            Some(ledger_url) => {
                let genesis_url = format!("{}/genesis", ledger_url);
                let body = reqwest::get(&genesis_url)
                    .await
                    .context("Failed to get genesis file from ledger")?
                    .text()
                    .await
                    .context("Failed to get the response text")?;
                let path = std::env::current_dir()
                    .context("Failed to obtain the current directory path")?
                    .join("resource")
                    .join("genesis_file.txn");
                let mut f = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(path.clone())
                    .context("Unable to open file")?;
                f.write_all(body.as_bytes()).context("Unable to write data")?;
                path.to_str()
                    .map(|s| s.to_string())
                    .ok_or(anyhow!("Failed to convert genesis file path to string".to_string()))
            }
            None => std::env::current_dir()
                .context("Failed to obtain the current directory path")?
                .join("resource")
                .join("indypool.txn")
                .to_str()
                .map(|s| s.to_string())
                .ok_or(anyhow!("Failed to convert genesis file path to string".to_string())),
        },
    }
}
