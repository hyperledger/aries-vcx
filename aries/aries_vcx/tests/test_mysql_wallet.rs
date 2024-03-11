#[macro_use]
extern crate serde_json;

#[cfg(test)]
mod dbtests {
    use std::error::Error;

    use aries_vcx::global::settings;
    use aries_vcx_core::wallet::{
        base_wallet::{did_wallet::DidWallet, BaseWallet, ManageWallet},
        indy::indy_wallet_config::IndyWalletConfig,
    };
    use libvcx_logger::LibvcxDefaultLogger;

    #[tokio::test]
    #[ignore]
    async fn test_mysql_init_issuer_with_mysql_wallet() -> Result<(), Box<dyn Error>> {
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
        let config_wallet = IndyWalletConfig::builder()
            .wallet_name(format!("faber_wallet_{}", uuid::Uuid::new_v4()))
            .wallet_key(settings::DEFAULT_WALLET_KEY.into())
            .wallet_key_derivation(settings::WALLET_KDF_RAW.into())
            .wallet_type("mysql".into())
            .storage_config(storage_config)
            .storage_credentials(storage_credentials)
            .build();

        let wallet = config_wallet.create_wallet().await?;
        wallet.configure_issuer(enterprise_seed).await?;

        wallet.create_and_store_my_did(None, None).await?;

        wallet.close_wallet().await?;
        Ok(())
    }
}
