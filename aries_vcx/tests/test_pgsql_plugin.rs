#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use serde_json::Value;

use aries_vcx::handlers::connection::connection::Connection;
use aries_vcx::init::{init_issuer_config, open_as_main_wallet};
use aries_vcx::libindy::utils::wallet::{close_main_wallet, configure_issuer_wallet, create_wallet, WalletConfig};
use aries_vcx::settings;
use aries_vcx::utils::devsetup::{AGENCY_DID, AGENCY_ENDPOINT, AGENCY_VERKEY};
use aries_vcx::utils::provision::{AgentProvisionConfig, provision_cloud_agent};

use crate::pgwallet::wallet_plugin::{finish_loading_postgres, load_storage_library, serialize_storage_plugin_configuration};


fn _init_wallet(wallet_storage_config: &WalletStorageConfig) -> Result<(), String> {
    info!("_init_wallet >>> wallet_storage_config:\n{}", serde_json::to_string(wallet_storage_config).unwrap());
    match wallet_storage_config.xtype.as_ref() {
        Some(wallet_type) => {
            let (plugin_library_path_serialized,
                plugin_init_function_serialized,
                storage_config_serialized,
                storage_credentials_serialized)
                = serialize_storage_plugin_configuration(wallet_type,
                                                         &wallet_storage_config.config,
                                                         &wallet_storage_config.credentials,
                                                         &wallet_storage_config.plugin_library_path,
                                                         &wallet_storage_config.plugin_init_function)?;
            let lib = load_storage_library(&plugin_library_path_serialized, &plugin_init_function_serialized)?;
            if wallet_type == "postgres_storage" {
                finish_loading_postgres(lib, &storage_config_serialized, &storage_credentials_serialized)?;
            }
            info!("Successfully loaded wallet plugin {}.", wallet_type);
            Ok(())
        }
        None => {
            info!("Using default builtin IndySDK wallets.");
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WalletStorageConfig {
    // Wallet storage type for agents wallets
    #[serde(rename = "type")]
    pub xtype: Option<String>,
    // Optional to override default library path. Default value is determined based on value of
    // xtype and OS
    pub plugin_library_path: Option<String>,
    // Optional to override default storage initialization function. Default value is  determined
    // based on value of xtype and OS
    pub plugin_init_function: Option<String>,
    // Wallet storage config for agents wallets
    pub config: Option<Value>,
    // Wallet storage credentials for agents wallets
    pub credentials: Option<Value>,
}

#[test]
#[cfg(feature = "plugin_test")]
fn test_provision_cloud_agent_with_pgsql_wallet() {
    let storage_config = r#"
          {
            "config": {
              "url": "localhost:5432",
              "max_connections" : 90,
              "connection_timeout" : 30,
              "wallet_scheme": "MultiWalletSingleTableSharedPool"
            },
            "credentials": {
              "account": "postgres",
              "password": "mysecretpassword",
              "admin_acount": "postgres",
              "admin_password": "mysecretpassword"
            },
            "type": "postgres_storage"
          }"#;
    let storage_config: WalletStorageConfig = serde_json::from_str(storage_config).unwrap();
    info!("Init pgsql storage: Starting.");
    _init_wallet(&storage_config).unwrap();
    info!("Init pgsql storage: Finished.");
    let storage_credentials = storage_config.credentials.map(|a| serde_json::to_string(&a).unwrap()).clone();
    let storage_config = storage_config.config.map(|a| serde_json::to_string(&a).unwrap()).clone();

    settings::clear_config();
    let enterprise_seed = "000000000000000000000000Trustee1";
    let config_wallet = WalletConfig {
        wallet_name: format!("faber_wallet_{}", uuid::Uuid::new_v4().to_string()),
        wallet_key: settings::DEFAULT_WALLET_KEY.into(),
        wallet_key_derivation: settings::WALLET_KDF_RAW.into(),
        wallet_type: Some("postgres_storage".into()),
        storage_config,
        storage_credentials,
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
    assert_eq!(4, 4);
}