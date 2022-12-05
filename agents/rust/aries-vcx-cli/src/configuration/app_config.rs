use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};

use super::{init_config::InitConfig, kdf::KeyDerivationMethod};

#[derive(Debug)]
pub struct AppConfig {
    ledger_url: Option<String>,
    genesis_file: String,
    trustee_seed: String,
    port: u32,
    host: String,
    log_level: String,
    accept_taa: bool,
    wallet_key: String,
    wallet_kdf: KeyDerivationMethod,
    agent_name: String,
}

impl AppConfig {
    fn build(init_config: InitConfig, trustee_seed: String, genesis_file: String) -> Self {
        Self {
            ledger_url: init_config.ledger_url().cloned(),
            genesis_file,
            trustee_seed,
            port: init_config.port(),
            host: init_config.host().to_string(),
            log_level: init_config.log_level().to_string(),
            accept_taa: init_config.accept_taa(),
            wallet_key: init_config.wallet_key().to_string(),
            wallet_kdf: init_config.wallet_kdf().clone(),
            agent_name: init_config
                .agent_name()
                .cloned()
                .unwrap_or_else(|| Alphanumeric.sample_string(&mut thread_rng(), 32)),
        }
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

    pub fn agent_name(&self) -> &str {
        self.agent_name.as_ref()
    }

    pub fn wallet_key(&self) -> &str {
        self.wallet_key.as_ref()
    }

    pub fn wallet_kdf(&self) -> &KeyDerivationMethod {
        &self.wallet_kdf
    }

    pub fn genesis_file(&self) -> &str {
        self.genesis_file.as_ref()
    }

    pub fn trustee_seed(&self) -> &str {
        self.trustee_seed.as_ref()
    }

    pub fn ledger_url(&self) -> Option<&String> {
        self.ledger_url.as_ref()
    }
}

pub fn load_app_config(init_config: InitConfig, trustee_seed: String, genesis_file: String) -> AppConfig {
    AppConfig::build(init_config, trustee_seed, genesis_file)
}
