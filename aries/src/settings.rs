extern crate serde_json;
extern crate url;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{RwLockWriteGuard, RwLockReadGuard, RwLock};

use indy_sys::INVALID_WALLET_HANDLE;
use serde_json::Value;
use strum::IntoEnumIterator;
use url::Url;

use agency_client::agency_settings;

use crate::error::prelude::*;
use crate::utils::{error, get_temp_dir_path};
use crate::utils::file::read_file;
use crate::utils::validation;
use crate::agency_client::agency_client::AgencyClient;

pub static CONFIG_POOL_NAME: &str = "pool_name";
pub static CONFIG_SDK_TO_REMOTE_ROLE: &str = "sdk_to_remote_role";
pub static CONFIG_INSTITUTION_DID: &str = "institution_did";
pub static CONFIG_INSTITUTION_VERKEY: &str = "institution_verkey";
// functionally not used
pub static CONFIG_WEBHOOK_URL: &str = "webhook_url";
pub static CONFIG_ENABLE_TEST_MODE: &str = "enable_test_mode";
pub static CONFIG_GENESIS_PATH: &str = "genesis_path";
pub static CONFIG_LOG_CONFIG: &str = "log_config";
pub static CONFIG_LINK_SECRET_ALIAS: &str = "link_secret_alias";
pub static CONFIG_EXPORTED_WALLET_PATH: &str = "exported_wallet_path";
pub static CONFIG_WALLET_BACKUP_KEY: &str = "backup_key";
pub static CONFIG_WALLET_KEY: &str = "wallet_key";
pub static CONFIG_WALLET_NAME: &'static str = "wallet_name";
pub static CONFIG_WALLET_TYPE: &'static str = "wallet_type";
pub static CONFIG_WALLET_KEY_DERIVATION: &'static str = "wallet_key_derivation";
pub static CONFIG_PROTOCOL_VERSION: &'static str = "protocol_version";
pub static CONFIG_PAYMENT_METHOD: &'static str = "payment_method";
pub static CONFIG_TXN_AUTHOR_AGREEMENT: &'static str = "author_agreement";
pub static CONFIG_POOL_CONFIG: &'static str = "pool_config";
pub static CONFIG_DID_METHOD: &str = "did_method";
// proprietary or aries
pub static CONFIG_ACTORS: &str = "actors";

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
#[cfg(not(target_os = "macos"))]
pub static DEFAULT_PAYMENT_PLUGIN: &str = "libnullpay.so";
#[cfg(target_os = "macos")]
pub static DEFAULT_PAYMENT_PLUGIN: &str = "libnullpay.dylib";
pub static DEFAULT_PAYMENT_INIT_FUNCTION: &str = "nullpay_init";
pub static DEFAULT_PAYMENT_METHOD: &str = "null";

lazy_static! {
    static ref SETTINGS: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
    pub static ref AGENCY_CLIENT: RwLock<AgencyClient> = RwLock::new(AgencyClient::default());
}

trait ToString {
    fn to_string(&self) -> String;
}

impl ToString for HashMap<String, String> {
    fn to_string(&self) -> String {
        let mut v = self.clone();
        v.insert(CONFIG_WALLET_KEY.to_string(), MASK_VALUE.to_string());
        serde_json::to_string(&v).unwrap()
    }
}

pub fn get_agency_client_mut() -> VcxResult<RwLockWriteGuard<'static, AgencyClient>> {
    let agency_client = AGENCY_CLIENT.write()?;
    Ok(agency_client)
}

pub fn get_agency_client() -> VcxResult<RwLockReadGuard<'static, AgencyClient>> {
    let agency_client = AGENCY_CLIENT.read()?;
    Ok(agency_client)
}

pub fn set_testing_defaults() -> u32 {
    trace!("set_testing_defaults >>>");

    // if this fails the program should exit
    let mut settings = SETTINGS.write().unwrap();

    settings.insert(CONFIG_POOL_NAME.to_string(), DEFAULT_POOL_NAME.to_string());
    settings.insert(CONFIG_INSTITUTION_DID.to_string(), DEFAULT_DID.to_string());
    settings.insert(CONFIG_LINK_SECRET_ALIAS.to_string(), DEFAULT_LINK_SECRET_ALIAS.to_string());
    settings.insert(CONFIG_PROTOCOL_VERSION.to_string(), DEFAULT_PROTOCOL_VERSION.to_string());
    settings.insert(CONFIG_PAYMENT_METHOD.to_string(), DEFAULT_PAYMENT_METHOD.to_string());
    settings.insert(CONFIG_WALLET_BACKUP_KEY.to_string(), DEFAULT_WALLET_BACKUP_KEY.to_string());

    get_agency_client_mut().unwrap().set_testing_defaults_agency();
    error::SUCCESS.code_num
}

pub fn indy_mocks_enabled() -> bool {
    let config = SETTINGS.read().unwrap();

    match config.get(CONFIG_ENABLE_TEST_MODE) {
        None => false,
        Some(value) => value == "true" || value == "indy"
    }
}

pub fn get_config_value(key: &str) -> VcxResult<String> {
    trace!("get_config_value >>> key: {}", key);

    SETTINGS
        .read()
        .or(Err(VcxError::from_msg(VcxErrorKind::InvalidConfiguration, "Cannot read settings")))?
        .get(key)
        .map(|v| v.to_string())
        .ok_or(VcxError::from_msg(VcxErrorKind::InvalidConfiguration, format!("Cannot read \"{}\" from settings", key)))
}

pub fn set_config_value(key: &str, value: &str) {
    trace!("set_config_value >>> key: {}, value: {}", key, value);
    SETTINGS
        .write().unwrap()
        .insert(key.to_string(), value.to_string());
}

pub fn get_protocol_version() -> usize {
    let protocol_version = match get_config_value(CONFIG_PROTOCOL_VERSION) {
        Ok(ver) => ver.parse::<usize>().unwrap_or_else(|err| {
            warn!("Can't parse value of protocol version from config ({}), use default one ({})", err, DEFAULT_PROTOCOL_VERSION);
            DEFAULT_PROTOCOL_VERSION
        }),
        Err(err) => {
            info!("Can't fetch protocol version from config ({}), use default one ({})", err, DEFAULT_PROTOCOL_VERSION);
            DEFAULT_PROTOCOL_VERSION
        }
    };
    if protocol_version > MAX_SUPPORTED_PROTOCOL_VERSION {
        error!("Protocol version from config {}, greater then maximal supported {}, use maximum one",
               protocol_version, MAX_SUPPORTED_PROTOCOL_VERSION);
        MAX_SUPPORTED_PROTOCOL_VERSION
    } else {
        protocol_version
    }
}

pub fn get_payment_method() -> String {
    get_config_value(CONFIG_PAYMENT_METHOD).unwrap_or(DEFAULT_PAYMENT_METHOD.to_string())
}

pub fn get_actors() -> Vec<Actors> {
    get_config_value(CONFIG_ACTORS)
        .and_then(|actors|
            serde_json::from_str(&actors)
                .map_err(|_| VcxError::from(VcxErrorKind::InvalidOption))
        ).unwrap_or_else(|_| Actors::iter().collect())
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq, EnumIter)]
#[serde(rename_all = "lowercase")]
pub enum Actors {
    Inviter,
    Invitee,
    Issuer,
    Holder,
    Prover,
    Verifier,
    Sender,
    Receiver,
}

pub fn clear_config() {
    trace!("clear_config >>>");
    let mut config = SETTINGS.write().unwrap();
    let mut agency_client = AGENCY_CLIENT.write().unwrap();
    config.clear();
    *agency_client = AgencyClient::default();
}

#[cfg(test)]
pub mod tests {
    use crate::utils::devsetup::{SetupDefaults, TempFile};

    use super::*;

    fn _pool_config() -> String {
        r#"{"timeout":40}"#.to_string()
    }

    fn base_config() -> serde_json::Value {
        json!({
            "pool_name" : "pool1",
            "config_name":"config1",
            "wallet_name":"test_read_config_file",
            "remote_to_sdk_did" : "UJGjM6Cea2YVixjWwHN9wq",
            "sdk_to_remote_did" : "AB3JM851T4EQmhh8CdagSP",
            "sdk_to_remote_verkey" : "888MFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
            "agency_verkey" : "91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
            "remote_to_sdk_verkey" : "91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
            "genesis_path":"/tmp/pool1.txn",
            "wallet_key":"key",
            "pool_config": _pool_config(),
            "payment_method": "null"
        })
    }

    fn config_json() -> String {
        base_config().to_string()
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_read_config_file() {
        let _setup = SetupDefaults::init();

        let mut config_file: TempFile = TempFile::create("test_init.json");
        config_file.write(&config_json());

        assert_eq!(read_file(&config_file.path).unwrap(), config_json());
    }

    fn _mandatory_config() -> HashMap<String, String> {
        let mut config: HashMap<String, String> = HashMap::new();
        config.insert(CONFIG_WALLET_KEY.to_string(), "password".to_string());
        config
    }


    #[test]
    #[cfg(feature = "general_test")]
    fn test_get_and_set_values() {
        let _setup = SetupDefaults::init();

        let key = "key1".to_string();
        let value1 = "value1".to_string();

        // Fails with invalid key
        assert_eq!(get_config_value(&key).unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);

        set_config_value(&key, &value1);
        assert_eq!(get_config_value(&key).unwrap(), value1);
    }
}
