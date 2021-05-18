use serde::Deserialize;

use indy::WalletHandle;
use agency_client::agent_utils;

use crate::error::prelude::*;
use crate::libindy::utils::{anoncreds, signus, wallet};
use crate::settings;
use crate::libindy::utils::wallet::{WalletConfig, IssuerConfig, configure_issuer_wallet};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    agency_url: String,
    pub agency_did: String,
    agency_verkey: String,
    wallet_name: Option<String>,
    wallet_key: String,
    wallet_type: Option<String>,
    agent_seed: Option<String>,
    enterprise_seed: Option<String>,
    wallet_key_derivation: Option<String>,
    name: Option<String>,
    path: Option<String>,
    storage_config: Option<String>,
    storage_credentials: Option<String>,
    pool_config: Option<String>,
    did_method: Option<String>,
    webhook_url: Option<String>,
    use_latest_protocols: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProvisionAgentConfig {
    pub agency_did: String,
    pub agency_verkey: String,
    pub agency_endpoint: String,
    pub agent_seed: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgencyConfig {
    pub agency_did: String,
    pub agency_endpoint: String,
    pub agency_verkey: String,
    pub remote_to_sdk_did: String,
    pub remote_to_sdk_verkey: String,
    pub sdk_to_remote_did: String,
    pub sdk_to_remote_verkey: String,
}

pub fn parse_config(config: &str) -> VcxResult<Config> {
    let my_config: Config = ::serde_json::from_str(&config)
        .map_err(|err|
            VcxError::from_msg(
                VcxErrorKind::InvalidConfiguration,
                format!("Cannot parse config: {}", err),
            )
        )?;
    Ok(my_config)
}

pub fn get_or_default(config: &Option<String>, default: &str) -> String {
    config.to_owned().unwrap_or(default.to_string())
}

pub fn set_config_values(my_config: &Config) {
    let wallet_name = get_or_default(&my_config.wallet_name, settings::DEFAULT_WALLET_NAME);

    settings::set_config_value(settings::CONFIG_WALLET_NAME, &wallet_name);
    settings::set_config_value(settings::CONFIG_WALLET_KEY, &my_config.wallet_key);
    settings::get_agency_client_mut().unwrap().set_agency_url(&my_config.agency_url);
    settings::get_agency_client_mut().unwrap().set_agency_did(&my_config.agency_did);
    settings::get_agency_client_mut().unwrap().set_agency_vk(&my_config.agency_verkey);
    settings::get_agency_client_mut().unwrap().set_agent_vk(&my_config.agency_verkey);

    settings::set_opt_config_value(settings::CONFIG_WALLET_KEY_DERIVATION, &my_config.wallet_key_derivation);
    settings::set_opt_config_value(settings::CONFIG_WALLET_TYPE, &my_config.wallet_type);
    settings::set_opt_config_value(settings::CONFIG_WALLET_STORAGE_CONFIG, &my_config.storage_config);
    settings::set_opt_config_value(settings::CONFIG_WALLET_STORAGE_CREDS, &my_config.storage_credentials);
    settings::set_opt_config_value(settings::CONFIG_POOL_CONFIG, &my_config.pool_config);
    settings::set_opt_config_value(settings::CONFIG_DID_METHOD, &my_config.did_method);
    settings::set_opt_config_value(settings::CONFIG_WEBHOOK_URL, &my_config.webhook_url);
}

fn _create_issuer_keys(my_did: &str, my_vk: &str, my_config: &Config) -> VcxResult<(String, String)> {
    if my_config.enterprise_seed == my_config.agent_seed {
        Ok((my_did.to_string(), my_vk.to_string()))
    } else {
        signus::create_and_store_my_did(
            my_config.enterprise_seed.as_ref().map(String::as_str),
            my_config.did_method.as_ref().map(String::as_str),
        )
    }
}

pub fn provision_cloud_agent(provision_agent_config: &ProvisionAgentConfig) -> VcxResult<AgencyConfig> {
    let (my_did, my_vk) = signus::create_and_store_my_did(provision_agent_config.agent_seed.as_ref().map(String::as_str), None)?;

    settings::get_agency_client_mut().unwrap().set_agency_did(&provision_agent_config.agency_did);
    settings::get_agency_client_mut().unwrap().set_agency_vk(&provision_agent_config.agency_verkey);
    settings::get_agency_client_mut().unwrap().set_agency_url(&provision_agent_config.agency_endpoint);
    settings::get_agency_client_mut().unwrap().set_my_vk(&my_vk);
    settings::get_agency_client_mut().unwrap().set_my_pwdid(&my_did);
    settings::get_agency_client_mut().unwrap().set_agent_vk(&provision_agent_config.agency_verkey); // This is reset when connection is established and agent did needs not be set before onboarding

    let (agent_did, agent_vk) = agent_utils::onboarding(&my_did, &my_vk, &provision_agent_config.agency_did)?;

    Ok(AgencyConfig {
        agency_did: provision_agent_config.agency_did.clone(),
        agency_endpoint: provision_agent_config.agency_endpoint.clone(),
        agency_verkey: provision_agent_config.agency_verkey.clone(),
        remote_to_sdk_did: agent_did,
        remote_to_sdk_verkey: agent_vk,
        sdk_to_remote_did:  my_did,
        sdk_to_remote_verkey: my_vk,
    })
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::api::vcx::vcx_shutdown;
    use crate::utils::devsetup::{SetupDefaults, SetupMocks};

    use super::*;
    use crate::settings::WALLET_KDF_RAW;
    use crate::utils::devsetup_agent::test::Alice;
    use serde_json::Value;


    #[test]
    #[cfg(feature = "general_test")]
    fn test_connect_register_provision() {
        let _setup = SetupMocks::init();
        let consumer = Alice::setup();

        assert_eq!(consumer.config_agency.agency_did, "VsKV7grR1BUE29mG2Fm2kX");
        assert_eq!(consumer.config_agency.agency_endpoint, "http://localhost:8080");
        assert_eq!(consumer.config_agency.agency_verkey, "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR");
        assert_eq!(consumer.config_wallet.wallet_key, "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY");
        assert_eq!(consumer.config_wallet.wallet_key_derivation,  "RAW");
    }
}
