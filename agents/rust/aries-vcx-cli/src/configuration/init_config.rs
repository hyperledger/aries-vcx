use std::path::Path;

use ::config as configrs;
use anyhow::Context;
use configrs::builder::DefaultState;
use serde::Deserialize;

use super::{app_args::AppArgs, kdf::KeyDerivationMethod};

const DEFAULT_LOG_LEVEL: &str = "warn";

#[derive(Deserialize, Clone)]
pub struct InitConfig {
    ledger_url: Option<String>,
    genesis_file: Option<String>,
    trustee_seed: Option<String>,
    port: u32,
    host: String,
    log_level: String,
    accept_taa: bool,
    wallet_key: String,
    wallet_kdf: KeyDerivationMethod,
    agent_name: Option<String>,
}

impl InitConfig {
    pub fn ledger_url(&self) -> Option<&String> {
        self.ledger_url.as_ref()
    }

    pub fn trustee_seed(&self) -> Option<&String> {
        self.trustee_seed.as_ref()
    }

    pub fn genesis_file(&self) -> Option<&String> {
        self.genesis_file.as_ref()
    }

    pub fn port(&self) -> u32 {
        self.port
    }

    pub fn host(&self) -> &str {
        self.host.as_ref()
    }

    pub fn log_level(&self) -> &str {
        self.log_level.as_ref()
    }

    pub fn accept_taa(&self) -> bool {
        self.accept_taa
    }

    pub fn wallet_key(&self) -> &str {
        self.wallet_key.as_ref()
    }

    pub fn wallet_kdf(&self) -> &KeyDerivationMethod {
        &self.wallet_kdf
    }

    pub fn agent_name(&self) -> Option<&String> {
        self.agent_name.as_ref()
    }
}

fn merge_app_args(config: configrs::ConfigBuilder<DefaultState>, app_args: &AppArgs) -> anyhow::Result<configrs::ConfigBuilder<DefaultState>> {
    Ok(config
        .set_override_option("port", app_args.port()).unwrap()
        .set_override_option("log_level", app_args.log_level()).unwrap()
        .set_override_option("agent_name", app_args.agent_name()).unwrap())
}

fn default_log_level() -> String {
    std::env::var("RUST_LOG").unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string())
}

pub fn load_init_config(args: &AppArgs) -> anyhow::Result<InitConfig> {
    println!("Initial app configuration will be loaded from {}", args.config_file());

    let config = configrs::Config::builder()
        .set_default("log_level", default_log_level()).context("Failed to set default log level")?
        .add_source(configrs::File::from(Path::new(&args.config_file())));

    let config = merge_app_args(config, args)?;

    config
        .build()
        .context(anyhow!("Failed to build configuration"))?
        .try_deserialize::<InitConfig>()
        .context("Failed to deserialize configuration")
}
