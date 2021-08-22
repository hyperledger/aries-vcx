mod pgwallet;

#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

#[cfg(test)]
#[cfg(feature = "plugin_test")]
mod test {
    use serde_json::Value;

    use aries_vcx::handlers::connection::connection::Connection;
    use aries_vcx::init::{init_issuer_config, open_as_main_wallet};
    use aries_vcx::libindy::utils::wallet::{close_main_wallet, configure_issuer_wallet, create_wallet, WalletConfig};
    use aries_vcx::{settings, libindy};
    use aries_vcx::utils::devsetup::{AGENCY_DID, AGENCY_ENDPOINT, AGENCY_VERKEY};
    use aries_vcx::utils::provision::{AgentProvisionConfig, provision_cloud_agent};

    use crate::pgwallet::wallet_plugin_loader::{PluginInitConfig, init_wallet_plugin};
    use std::env;
    use aries_vcx::utils::test_logger::LibvcxDefaultLogger;

    #[test]
    fn test_provision_cloud_agent_with_pgsql_wallet() {
        LibvcxDefaultLogger::init_testing_logger();
        let storage_config = r#"
          {
              "url": "localhost:5432",
              "max_connections" : 90,
              "connection_timeout" : 30,
              "wallet_scheme": "MultiWalletSingleTableSharedPool"
          }"#;
        let storage_credentials = r#"
          {
              "account": "postgres",
              "password": "mysecretpassword",
              "admin_account": "postgres",
              "admin_password": "mysecretpassword"
          }"#;
        let init_config: PluginInitConfig = PluginInitConfig {
            storage_type: String::from("postgres_storage"),
            plugin_library_path: None,
            plugin_init_function: None,
            config: storage_config.into(),
            credentials: storage_credentials.into()
        };

        init_wallet_plugin(&init_config).unwrap();

        let enterprise_seed = "000000000000000000000000Trustee1";
        let config_wallet = WalletConfig {
            wallet_name: format!("faber_wallet_{}", uuid::Uuid::new_v4().to_string()),
            wallet_key: settings::DEFAULT_WALLET_KEY.into(),
            wallet_key_derivation: settings::WALLET_KDF_RAW.into(),
            wallet_type: Some("postgres_storage".into()),
            storage_config: Some(String::from(storage_config)),
            storage_credentials: Some(String::from(storage_credentials)),
            rekey: None,
            rekey_derivation_method: None,
        };
        let config_provision_agent = AgentProvisionConfig {
            agency_did: AGENCY_DID.to_string(),
            agency_verkey: AGENCY_VERKEY.to_string(),
            agency_endpoint: AGENCY_ENDPOINT.to_string(),
            agent_seed: None,
        };
        create_wallet(&config_wallet).unwrap();
        open_as_main_wallet(&config_wallet).unwrap();
        let config_issuer = configure_issuer_wallet(enterprise_seed).unwrap();
        init_issuer_config(&config_issuer).unwrap();
        provision_cloud_agent(&config_provision_agent).unwrap();
        close_main_wallet().unwrap();
    }
}