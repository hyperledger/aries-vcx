#[macro_use]
extern crate serde_json;

#[cfg(test)]
mod dbtests {
    use std::error::Error;

    use aries_vcx::global::settings;
    use aries_vcx_core::wallet::{
        base_wallet::DidWallet,
        indy::{
            wallet::{close_wallet, create_and_open_wallet, wallet_configure_issuer},
            IndySdkWallet, WalletConfig,
        },
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
        let config_wallet = WalletConfig::builder()
            .wallet_name(format!("faber_wallet_{}", uuid::Uuid::new_v4()))
            .wallet_key(settings::DEFAULT_WALLET_KEY.into())
            .wallet_key_derivation(settings::WALLET_KDF_RAW.into())
            .wallet_type("mysql".into())
            .storage_config(storage_config)
            .storage_credentials(storage_credentials)
            .build();

        let wallet_handle = create_and_open_wallet(&config_wallet).await?;
        let _config_issuer = wallet_configure_issuer(wallet_handle, enterprise_seed).await?;

        IndySdkWallet::new(wallet_handle)
            .create_and_store_my_did(None, None)
            .await?;
        close_wallet(wallet_handle).await?;
        Ok(())
    }
}
