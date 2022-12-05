use std::convert::TryFrom;

use anyhow::anyhow;
use aries_vcx_agent::{Agent as AriesAgent, InitConfig, PoolInitConfig, WalletInitConfig};

use crate::configuration::AppConfig;

use super::CliAriesAgent;

impl TryFrom<&AppConfig> for InitConfig {
    type Error = anyhow::Error;

    fn try_from(config: &AppConfig) -> Result<Self, Self::Error> {
        let enterprise_seed = config.trustee_seed().to_string();
        let agent_name = config.agent_name().to_string();
        let genesis_path = config.genesis_file().to_string();
        Ok(InitConfig {
            enterprise_seed,
            pool_config: PoolInitConfig {
                genesis_path,
                pool_name: format!("pool_{}", agent_name),
            },
            wallet_config: WalletInitConfig {
                wallet_name: format!("wallet_{}", agent_name),
                wallet_key: config.wallet_key().to_string(),
                wallet_kdf: config.wallet_kdf().to_string(),
            },
            agency_config: None,
            service_endpoint: format!("http://{}:{}/didcomm", config.host(), config.port()),
        })
    }
}

pub async fn initialize_agent(config: &AppConfig) -> anyhow::Result<CliAriesAgent> {
    let agent_config: InitConfig = config.try_into()?;
    let aries_agent = AriesAgent::initialize(agent_config)
        .await
        .map_err(|err| anyhow!("Agent initialization failed: {}", err.to_string()))?;
    Ok(CliAriesAgent::new(aries_agent))
}
