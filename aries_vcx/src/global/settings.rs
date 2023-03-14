use std::{collections::HashMap, sync::RwLock};

use crate::{errors::error::prelude::*, indy::wallet::IssuerConfig};

pub static CONFIG_POOL_NAME: &str = "pool_name";
pub static CONFIG_SDK_TO_REMOTE_ROLE: &str = "sdk_to_remote_role";
pub static CONFIG_INSTITUTION_DID: &str = "institution_did";
pub static CONFIG_INSTITUTION_VERKEY: &str = "institution_verkey";

// functionally not used
pub static CONFIG_WEBHOOK_URL: &str = "webhook_url";
pub static CONFIG_ENABLE_TEST_MODE: &str = "enable_test_mode";
pub static CONFIG_GENESIS_PATH: &str = "genesis_path";
pub static CONFIG_LOG_CONFIG: &str = "log_config";
pub static CONFIG_EXPORTED_WALLET_PATH: &str = "exported_wallet_path";
pub static CONFIG_WALLET_BACKUP_KEY: &str = "backup_key";
pub static CONFIG_WALLET_KEY: &str = "wallet_key";
pub static CONFIG_WALLET_NAME: &str = "wallet_name";
pub static CONFIG_WALLET_TYPE: &str = "wallet_type";
pub static CONFIG_WALLET_KEY_DERIVATION: &str = "wallet_key_derivation";
pub static CONFIG_PROTOCOL_VERSION: &str = "protocol_version";
pub static CONFIG_TXN_AUTHOR_AGREEMENT: &str = "author_agreement";
pub static CONFIG_POOL_CONFIG: &str = "pool_config";
pub static CONFIG_DID_METHOD: &str = "did_method";
pub static DEFAULT_PROTOCOL_VERSION: usize = 2;
pub static MAX_SUPPORTED_PROTOCOL_VERSION: usize = 2;
pub static UNINITIALIZED_WALLET_KEY: &str = "<KEY_IS_NOT_SET>";
pub static DEFAULT_GENESIS_PATH: &str = "genesis.txn";
pub static DEFAULT_WALLET_NAME: &str = "LIBVCX_SDK_WALLET";
pub static DEFAULT_POOL_NAME: &str = "pool1";
pub static DEFAULT_LINK_SECRET_ALIAS: &str = "main";
pub static DEFAULT_DID: &str = "2hoqvcwupRTUNkXn6ArYzs";
pub static DEFAULT_ROLE: &str = "0";
pub static DEFAULT_WALLET_BACKUP_KEY: &str = "backup_wallet_key";
pub static DEFAULT_WALLET_KEY: &str = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY";
pub static MASK_VALUE: &str = "********";
pub static WALLET_KDF_RAW: &str = "RAW";
pub static WALLET_KDF_ARGON2I_INT: &str = "ARGON2I_INT";
pub static WALLET_KDF_ARGON2I_MOD: &str = "ARGON2I_MOD";
pub static WALLET_KDF_DEFAULT: &str = WALLET_KDF_ARGON2I_MOD;

lazy_static! {
    static ref SETTINGS: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

pub fn enable_indy_mocks() -> VcxResult<()> {
    debug!("enable_indy_mocks >>>");
    set_config_value(CONFIG_ENABLE_TEST_MODE, "true")
}

pub fn disable_indy_mocks() -> VcxResult<()> {
    debug!("disable_indy_mocks >>>");
    set_config_value(CONFIG_ENABLE_TEST_MODE, "false")
}

pub fn indy_mocks_enabled() -> bool {
    let config = SETTINGS.read().expect("Unable to access SETTINGS");

    match config.get(CONFIG_ENABLE_TEST_MODE) {
        None => false,
        Some(value) => {
            debug!("indy_mocks_enabled >>> {}", value);
            value == "true" || value == "indy"
        }
    }
}

pub fn get_config_value(key: &str) -> VcxResult<String> {
    trace!("get_config_value >>> key: {}", key);

    SETTINGS
        .read()
        .or(Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidConfiguration,
            "Cannot read settings",
        )))?
        .get(key)
        .map(|v| v.to_string())
        .ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidConfiguration,
            format!("Cannot read \"{}\" from settings", key),
        ))
}

pub fn set_config_value(key: &str, value: &str) -> VcxResult<()> {
    trace!("set_config_value >>> key: {}, value: {}", key, value);
    SETTINGS
        .write()
        .or(Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::UnknownError,
            "Cannot write settings",
        )))?
        .insert(key.to_string(), value.to_string());
    Ok(())
}

pub fn reset_config_values() -> VcxResult<()> {
    trace!("reset_config_values >>>");
    let mut config = SETTINGS.write()?;
    config.clear();
    Ok(())
}

pub fn set_test_configs() -> String {
    trace!("set_testing_defaults >>>");
    let mut settings = SETTINGS
        .write()
        .expect("Unabled to access SETTINGS while setting test configs");
    let institution_did = CONFIG_INSTITUTION_DID;
    settings.insert(CONFIG_POOL_NAME.to_string(), DEFAULT_POOL_NAME.to_string());
    settings.insert(institution_did.to_string(), DEFAULT_DID.to_string());
    settings.insert(
        CONFIG_PROTOCOL_VERSION.to_string(),
        DEFAULT_PROTOCOL_VERSION.to_string(),
    );
    settings.insert(
        CONFIG_WALLET_BACKUP_KEY.to_string(),
        DEFAULT_WALLET_BACKUP_KEY.to_string(),
    );
    institution_did.to_string()
}

pub fn get_protocol_version() -> usize {
    let protocol_version = match get_config_value(CONFIG_PROTOCOL_VERSION) {
        Ok(ver) => ver.parse::<usize>().unwrap_or_else(|err| {
            warn!(
                "Can't parse value of protocol version from config ({}), use default one ({})",
                err, DEFAULT_PROTOCOL_VERSION
            );
            DEFAULT_PROTOCOL_VERSION
        }),
        Err(err) => {
            info!(
                "Can't fetch protocol version from config ({}), use default one ({})",
                err, DEFAULT_PROTOCOL_VERSION
            );
            DEFAULT_PROTOCOL_VERSION
        }
    };
    if protocol_version > MAX_SUPPORTED_PROTOCOL_VERSION {
        error!(
            "Protocol version from config {}, greater then maximal supported {}, use maximum one",
            protocol_version, MAX_SUPPORTED_PROTOCOL_VERSION
        );
        MAX_SUPPORTED_PROTOCOL_VERSION
    } else {
        protocol_version
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;
    use crate::utils::devsetup::SetupDefaults;

    fn _pool_config() -> String {
        r#"{"timeout":40}"#.to_string()
    }

    fn _mandatory_config() -> HashMap<String, String> {
        let mut config: HashMap<String, String> = HashMap::new();
        config.insert(CONFIG_WALLET_KEY.to_string(), "password".to_string());
        config
    }

    #[test]
    fn test_get_and_set_values() {
        let _setup = SetupDefaults::init();

        let key = "key1".to_string();
        let value1 = "value1".to_string();

        // Fails with invalid key
        assert_eq!(
            get_config_value(&key).unwrap_err().kind(),
            AriesVcxErrorKind::InvalidConfiguration
        );

        set_config_value(&key, &value1).unwrap();
        assert_eq!(get_config_value(&key).unwrap(), value1);
    }
}

pub fn init_issuer_config(config: &IssuerConfig) -> VcxResult<()> {
    set_config_value(CONFIG_INSTITUTION_DID, &config.institution_did)?;
    Ok(())
}
