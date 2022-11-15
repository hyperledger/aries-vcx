#[cfg(feature = "mysql_test")]
#[macro_use]
extern crate log;

#[cfg(feature = "mysql_test")]
#[macro_use]
extern crate serde_json;

#[cfg(feature = "mysql_test")]
extern crate sqlx;

#[cfg(feature = "mysql_test")]
mod test_utils {
    use sqlx::{Connection, MySqlConnection};

    pub async fn setup_mysql_walletdb() -> Result<String, sqlx::Error> {
        debug!("Running query.");
        let db_name = format!("mysqltest_{}", uuid::Uuid::new_v4().to_string()).replace("-", "_");
        let url = "mysql://root:mysecretpassword@localhost:3306";
        let mut connection = MySqlConnection::connect(url).await?;
        let query = format!("CREATE DATABASE {};", db_name);
        let query = sqlx::query(&query);
        let res = query.execute(&mut connection).await;
        debug!("Create database result: {:?}", res);
        connection.close().await.unwrap();

        let url = format!("mysql://root:mysecretpassword@localhost:3306/{}", db_name);
        let mut connection = MySqlConnection::connect(&url).await?;
        let res = sqlx::migrate!("./migrations").run(&mut connection).await;
        debug!("Create tables result: {:?}", res);
        Ok(db_name)
    }
}

#[cfg(feature = "mysql_test")]
#[cfg(test)]
mod dbtests {
    use agency_client::agency_client::AgencyClient;
    use agency_client::configuration::AgentProvisionConfig;
    use aries_vcx::global::settings;
    use aries_vcx::global::settings::init_issuer_config;
    use aries_vcx::utils::devsetup::{AGENCY_DID, AGENCY_ENDPOINT, AGENCY_VERKEY};
    use aries_vcx::utils::provision::provision_cloud_agent;
    use aries_vcx::utils::test_logger::LibvcxDefaultLogger;

    use crate::test_utils::setup_mysql_walletdb;

    #[tokio::test]
    async fn test_provision_cloud_agent_with_mysql_wallet() {
        LibvcxDefaultLogger::init_testing_logger();
        let db_name = setup_mysql_walletdb().await.unwrap();
        let storage_config = json!({
            "read_host": "localhost",
            "write_host": "localhost",
            "port": 3306 as u32,
            "db_name": db_name,
            "default_connection_limit": 50 as u32
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
            agency_endpoint: AGENCY_ENDPOINT.to_string(),
            agent_seed: None,
        };
        // create_main_wallet(&config_wallet).await.unwrap();
        create_wallet_with_master_secret(&config_wallet).await.unwrap();
        let wallet_handle = open_wallet(&config_wallet).await.unwrap();
        let config_issuer = wallet_configure_issuer(wallet_handle, enterprise_seed).await.unwrap();
        init_issuer_config(&config_issuer).unwrap();
        let mut agency_client = AgencyClient::new();
        provision_cloud_agent(&mut agency_client, wallet_handle, &config_provision_agent)
            .await
            .unwrap();
        close_wallet(wallet_handle).await.unwrap();
    }
}
