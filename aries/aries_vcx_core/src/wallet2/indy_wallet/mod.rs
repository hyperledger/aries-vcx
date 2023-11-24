use crate::wallet::indy::IndySdkWallet;

use super::BaseWallet2;

pub mod indy_did_wallet;
pub mod indy_record_wallet;

const WALLET_OPTIONS: &str =
    r#"{"retrieveType": true, "retrieveValue": true, "retrieveTags": true}"#;

const SEARCH_OPTIONS: &str = r#"{"retrieveType": true, "retrieveValue": true, "retrieveTags": true, "retrieveRecords": true}"#;

impl BaseWallet2 for IndySdkWallet {}

pub(crate) mod test_helper {
    use crate::wallet::indy::{wallet::create_and_open_wallet, IndySdkWallet, WalletConfigBuilder};

    use serde_json::json;

    pub async fn create_test_wallet() -> IndySdkWallet {
        let default_wallet_key = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY";
        let wallet_kdf_raw = "RAW";

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
        let config_wallet = WalletConfigBuilder::default()
            .wallet_name(format!("faber_wallet_{}", uuid::Uuid::new_v4()))
            .wallet_key(default_wallet_key)
            .wallet_key_derivation(wallet_kdf_raw)
            .wallet_type("mysql")
            .storage_config(storage_config)
            .storage_credentials(storage_credentials)
            .build()
            .unwrap();

        let wallet_handle = create_and_open_wallet(&config_wallet).await.unwrap();

        IndySdkWallet::new(wallet_handle)
    }
}
