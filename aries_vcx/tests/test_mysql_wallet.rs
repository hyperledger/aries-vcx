#[macro_use]
extern crate serde_json;

#[cfg(test)]
mod dbtests {
    use std::sync::Arc;

    use agency_client::agency_client::AgencyClient;
    use agency_client::configuration::AgentProvisionConfig;
    use aries_vcx::global::settings;
    use aries_vcx::global::settings::init_issuer_config;
    use aries_vcx::utils::devsetup::{AGENCY_DID, AGENCY_ENDPOINT, AGENCY_VERKEY};
    use aries_vcx::utils::provision::provision_cloud_agent;
    use aries_vcx::utils::test_logger::LibvcxDefaultLogger;
    use aries_vcx_core::indy::wallet::{
        close_wallet, create_wallet_with_master_secret, open_wallet, wallet_configure_issuer, WalletConfig,
        WalletConfigBuilder,
    };
    use aries_vcx_core::wallet::indy_wallet::IndySdkWallet;

    #[tokio::test]
    #[ignore]
    async fn test_mysql_provision_cloud_agent_with_mysql_wallet() {
        LibvcxDefaultLogger::init_testing_logger();
        let db_name = format!("mysqltest_{}", uuid::Uuid::new_v4()).replace('-', "_");
        let storage_config = json!({
            "read_host": "localhost",
            "write_host": "localhost",
            "port": 3306,
            "db_name": db_name,
            "default_connection_limit": 50
        })
        .to_string();
        let storage_credentials = json!({
            "user": "root",
            "pass": "mysecretpassword"
        })
        .to_string();
        let enterprise_seed = "000000000000000000000000Trustee1";
        let config_wallet: WalletConfig = WalletConfigBuilder::default()
            .wallet_name(format!("faber_wallet_{}", uuid::Uuid::new_v4()))
            .wallet_key(settings::DEFAULT_WALLET_KEY)
            .wallet_key_derivation(settings::WALLET_KDF_RAW)
            .wallet_type("mysql")
            .storage_config(storage_config)
            .storage_credentials(storage_credentials)
            .build()
            .unwrap();
        let config_provision_agent: AgentProvisionConfig = AgentProvisionConfig {
            agency_did: AGENCY_DID.to_string(),
            agency_verkey: AGENCY_VERKEY.to_string(),
            agency_endpoint: AGENCY_ENDPOINT.parse().unwrap(),
            agent_seed: None,
        };
        
        create_wallet_with_master_secret(&config_wallet).await.unwrap();
        let wallet_handle = open_wallet(&config_wallet).await.unwrap();
        let profile = Arc::new(IndySdkWallet::new(wallet_handle));
        let config_issuer = wallet_configure_issuer(wallet_handle, enterprise_seed).await.unwrap();
        init_issuer_config(&config_issuer.institution_did).unwrap();
        let mut agency_client = AgencyClient::new();
        provision_cloud_agent(&mut agency_client, profile, &config_provision_agent)
            .await
            .unwrap();
        close_wallet(wallet_handle).await.unwrap();
    }
}
