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
pub static CONFIG_INSTITUTION_NAME: &str = "institution_name";
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
pub static CONFIG_WALLET_STORAGE_CONFIG: &'static str = "storage_config";
pub static CONFIG_WALLET_STORAGE_CREDS: &'static str = "storage_credentials";
pub static CONFIG_WALLET_HANDLE: &'static str = "wallet_handle";
pub static CONFIG_THREADPOOL_SIZE: &'static str = "threadpool_size";
pub static CONFIG_WALLET_KEY_DERIVATION: &'static str = "wallet_key_derivation";
pub static CONFIG_PROTOCOL_VERSION: &'static str = "protocol_version";
pub static CONFIG_PAYMENT_METHOD: &'static str = "payment_method";
pub static CONFIG_TXN_AUTHOR_AGREEMENT: &'static str = "author_agreement";
pub static CONFIG_USE_LATEST_PROTOCOLS: &'static str = "use_latest_protocols";
pub static CONFIG_POOL_CONFIG: &'static str = "pool_config";
pub static CONFIG_DID_METHOD: &str = "did_method";
// proprietary or aries
pub static CONFIG_ACTORS: &str = "actors";

pub static DEFAULT_PROTOCOL_VERSION: usize = 2;
pub static MAX_SUPPORTED_PROTOCOL_VERSION: usize = 2;
pub static UNINITIALIZED_WALLET_KEY: &str = "<KEY_IS_NOT_SET>";
pub static DEFAULT_GENESIS_PATH: &str = "genesis.txn";
pub static DEFAULT_EXPORTED_WALLET_PATH: &str = "wallet.txn";
pub static DEFAULT_WALLET_NAME: &str = "LIBVCX_SDK_WALLET";
pub static DEFAULT_POOL_NAME: &str = "pool1";
pub static DEFAULT_LINK_SECRET_ALIAS: &str = "main";
pub static DEFAULT_DEFAULT: &str = "default";
pub static DEFAULT_URL: &str = "http://127.0.0.1:8080";
pub static DEFAULT_DID: &str = "2hoqvcwupRTUNkXn6ArYzs";
pub static DEFAULT_VERKEY: &str = "FuN98eH2eZybECWkofW6A9BKJxxnTatBCopfUiNxo6ZB";
pub static DEFAULT_ROLE: &str = "0";
pub static DEFAULT_ENABLE_TEST_MODE: &str = "false";
pub static DEFAULT_WALLET_BACKUP_KEY: &str = "backup_wallet_key";
pub static DEFAULT_WALLET_KEY: &str = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY";
pub static DEFAULT_THREADPOOL_SIZE: usize = 8;
pub static MASK_VALUE: &str = "********";
pub static WALLET_KDF_RAW: &str = "RAW";
pub static WALLET_KDF_ARGON2I_INT: &str = "ARGON2I_INT";
pub static WALLET_KDF_DEFAULT: &str = WALLET_KDF_ARGON2I_INT;
#[cfg(not(target_os = "macos"))]
pub static DEFAULT_PAYMENT_PLUGIN: &str = "libnullpay.so";
#[cfg(target_os = "macos")]
pub static DEFAULT_PAYMENT_PLUGIN: &str = "libnullpay.dylib";
pub static DEFAULT_PAYMENT_INIT_FUNCTION: &str = "nullpay_init";
pub static DEFAULT_USE_LATEST_PROTOCOLS: &str = "false";
pub static DEFAULT_PAYMENT_METHOD: &str = "null";
pub static MAX_THREADPOOL_SIZE: usize = 128;
pub static MOCK_DEFAULT_INDY_PROOF_VALIDATION: &str = "true";

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
    settings.insert(CONFIG_WALLET_NAME.to_string(), DEFAULT_WALLET_NAME.to_string());
    settings.insert(CONFIG_WALLET_TYPE.to_string(), DEFAULT_DEFAULT.to_string());
    settings.insert(CONFIG_INSTITUTION_DID.to_string(), DEFAULT_DID.to_string());
    settings.insert(CONFIG_INSTITUTION_NAME.to_string(), DEFAULT_DEFAULT.to_string());
    settings.insert(CONFIG_WEBHOOK_URL.to_string(), DEFAULT_URL.to_string());
    settings.insert(CONFIG_SDK_TO_REMOTE_ROLE.to_string(), DEFAULT_ROLE.to_string());
    settings.insert(CONFIG_WALLET_KEY.to_string(), DEFAULT_WALLET_KEY.to_string());
    settings.insert(CONFIG_WALLET_KEY_DERIVATION.to_string(), WALLET_KDF_RAW.to_string());
    settings.insert(CONFIG_LINK_SECRET_ALIAS.to_string(), DEFAULT_LINK_SECRET_ALIAS.to_string());
    settings.insert(CONFIG_PROTOCOL_VERSION.to_string(), DEFAULT_PROTOCOL_VERSION.to_string());
    settings.insert(CONFIG_EXPORTED_WALLET_PATH.to_string(),
                    get_temp_dir_path(DEFAULT_EXPORTED_WALLET_PATH).to_str().unwrap_or("").to_string());
    settings.insert(CONFIG_WALLET_BACKUP_KEY.to_string(), DEFAULT_WALLET_BACKUP_KEY.to_string());
    settings.insert(CONFIG_THREADPOOL_SIZE.to_string(), DEFAULT_THREADPOOL_SIZE.to_string());
    settings.insert(CONFIG_PAYMENT_METHOD.to_string(), DEFAULT_PAYMENT_METHOD.to_string());
    settings.insert(CONFIG_USE_LATEST_PROTOCOLS.to_string(), DEFAULT_USE_LATEST_PROTOCOLS.to_string());

    get_agency_client_mut().unwrap().set_testing_defaults_agency();
    error::SUCCESS.code_num
}

pub fn validate_config(config: &HashMap<String, String>) -> VcxResult<u32> {
    trace!("validate_config >>> config: {:?}", config);

    //Mandatory parameters
    if crate::libindy::utils::wallet::get_wallet_handle() == INVALID_WALLET_HANDLE && config.get(CONFIG_WALLET_KEY).is_none() {
        return Err(VcxError::from(VcxErrorKind::MissingWalletKey));
    }

    // If values are provided, validate they're in the correct format
    validate_optional_config_val(config.get(CONFIG_INSTITUTION_DID), VcxErrorKind::InvalidDid, validation::validate_did)?;
    validate_optional_config_val(config.get(CONFIG_INSTITUTION_VERKEY), VcxErrorKind::InvalidVerkey, validation::validate_verkey)?;
    validate_optional_config_val(config.get(CONFIG_WEBHOOK_URL), VcxErrorKind::InvalidUrl, Url::parse)?;
    validate_optional_config_val(config.get(CONFIG_ACTORS), VcxErrorKind::InvalidOption, validation::validate_actors)?;

    get_agency_client()?.validate()?;
    Ok(error::SUCCESS.code_num)
}

fn validate_mandatory_config_val<F, S, E>(val: Option<&String>, err: VcxErrorKind, closure: F) -> VcxResult<u32>
    where F: Fn(&str) -> Result<S, E> {
    closure(val.as_ref().ok_or(VcxError::from(err))?)
        .or(Err(VcxError::from(err)))?;

    Ok(error::SUCCESS.code_num)
}

fn validate_optional_config_val<F, S, E>(val: Option<&String>, err: VcxErrorKind, closure: F) -> VcxResult<u32>
    where F: Fn(&str) -> Result<S, E> {
    if val.is_none() { return Ok(error::SUCCESS.code_num); }

    closure(val.as_ref().ok_or(VcxError::from(VcxErrorKind::InvalidConfiguration))?)
        .or(Err(VcxError::from(err)))?;

    Ok(error::SUCCESS.code_num)
}

pub fn validate_payment_method() -> VcxResult<u32> {
    validate_mandatory_config_val(get_config_value(CONFIG_PAYMENT_METHOD).ok().as_ref(),
                                  VcxErrorKind::MissingPaymentMethod, validation::validate_payment_method)
}

pub fn settings_as_string() -> String {
    SETTINGS.read().unwrap().to_string()
}

pub fn log_settings() {
    let settings = SETTINGS.read().unwrap();
    trace!("loaded settings: {:?}", settings.to_string());
}

pub fn indy_mocks_enabled() -> bool {
    let config = SETTINGS.read().unwrap();

    match config.get(CONFIG_ENABLE_TEST_MODE) {
        None => false,
        Some(value) => value == "true" || value == "indy"
    }
}

pub fn enable_mock_generate_indy_proof() {}

pub fn process_config_string(config: &str, do_validation: bool) -> VcxResult<u32> {
    trace!("process_config_string >>> config {}", config);

    let configuration: Value = serde_json::from_str(config)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot parse config: {}", err)))?;

    if let Value::Object(ref map) = configuration {
        for (key, value) in map {
            match value {
                Value::String(value_) => set_config_value(key, &value_),
                Value::Array(value_) => set_config_value(key, &json!(value_).to_string()),
                Value::Object(value_) => set_config_value(key, &json!(value_).to_string()),
                Value::Bool(value_) => set_config_value(key, &json!(value_).to_string()),
                _ => return Err(VcxError::from(VcxErrorKind::InvalidJson)),
            }
        }
    }

    // TODO: This won't be necessary - move to open wallet for now?
    get_agency_client_mut()?.process_config_string(config, false)?; // False due to failing tests

    if do_validation {
        let setting = SETTINGS.read()
            .or(Err(VcxError::from(VcxErrorKind::InvalidConfiguration)))?;
        validate_config(&setting.borrow())
    } else {
        Ok(error::SUCCESS.code_num)
    }
}

pub fn process_config_file(path: &str) -> VcxResult<u32> {
    trace!("process_config_file >>> path: {}", path);

    if !Path::new(path).is_file() {
        error!("Configuration path was invalid");
        Err(VcxError::from_msg(VcxErrorKind::InvalidConfiguration, "Cannot find config file"))
    } else {
        let config = read_file(path)?;
        process_config_string(&config, true)
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

pub fn get_wallet_name() -> VcxResult<String> {
    get_config_value(CONFIG_WALLET_NAME)
        .map_err(|_| VcxError::from(VcxErrorKind::MissingWalletKey))
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

pub fn get_opt_config_value(key: &str) -> Option<String> {
    trace!("get_opt_config_value >>> key: {}", key);
    match SETTINGS.read() {
        Ok(x) => x,
        Err(_) => return None
    }
        .get(key)
        .map(|v| v.to_string())
}

pub fn set_opt_config_value(key: &str, value: &Option<String>) {
    if let Some(v) = value {
        set_config_value(key, v.as_str())
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

    fn _institution_name() -> String {
        "enterprise".to_string()
    }

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
            "institution_name" : _institution_name(),
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
    fn test_bad_path() {
        let _setup = SetupDefaults::init();

        let path = "garbage.txt";
        assert_eq!(process_config_file(&path).unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_read_config_file() {
        let _setup = SetupDefaults::init();

        let mut config_file: TempFile = TempFile::create("test_init.json");
        config_file.write(&config_json());

        assert_eq!(read_file(&config_file.path).unwrap(), config_json());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_process_file() {
        let _setup = SetupDefaults::init();

        let mut config_file: TempFile = TempFile::create("test_init.json");
        config_file.write(&config_json());

        assert_eq!(process_config_file(&config_file.path).unwrap(), error::SUCCESS.code_num);

        assert_eq!(get_config_value("institution_name").unwrap(), _institution_name());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_process_config_str() {
        let _setup = SetupDefaults::init();

        assert_eq!(process_config_string(&config_json(), true).unwrap(), error::SUCCESS.code_num);

        assert_eq!(get_config_value("institution_name").unwrap(), _institution_name());
        assert_eq!(get_config_value("pool_config").unwrap(), _pool_config());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_validate_config() {
        let _setup = SetupDefaults::init();

        let config: HashMap<String, String> = serde_json::from_str(&config_json()).unwrap();
        assert_eq!(validate_config(&config).unwrap(), error::SUCCESS.code_num);
    }

    fn _mandatory_config() -> HashMap<String, String> {
        let mut config: HashMap<String, String> = HashMap::new();
        config.insert(CONFIG_WALLET_KEY.to_string(), "password".to_string());
        config
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_validate_config_failures() {
        let _setup = SetupDefaults::init();

        let invalid = "invalid";

        let config = HashMap::new();
        assert_eq!(validate_config(&config).unwrap_err().kind(), VcxErrorKind::MissingWalletKey);

        let mut config = _mandatory_config();
        config.insert(CONFIG_INSTITUTION_DID.to_string(), invalid.to_string());
        assert_eq!(validate_config(&config).unwrap_err().kind(), VcxErrorKind::InvalidDid);

        let mut config = _mandatory_config();
        config.insert(CONFIG_INSTITUTION_VERKEY.to_string(), invalid.to_string());
        assert_eq!(validate_config(&config).unwrap_err().kind(), VcxErrorKind::InvalidVerkey);

        let mut config = _mandatory_config();
        config.insert(CONFIG_WEBHOOK_URL.to_string(), invalid.to_string());
        assert_eq!(validate_config(&config).unwrap_err().kind(), VcxErrorKind::InvalidUrl);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_validate_optional_config_val() {
        let _setup = SetupDefaults::init();

        let closure = Url::parse;
        let mut config: HashMap<String, String> = HashMap::new();
        config.insert("valid".to_string(), DEFAULT_URL.to_string());
        config.insert("invalid".to_string(), "invalid_url".to_string());

        //Success
        assert_eq!(validate_optional_config_val(config.get("valid"), VcxErrorKind::InvalidUrl, closure).unwrap(),
                   error::SUCCESS.code_num);

        // Success with No config
        assert_eq!(validate_optional_config_val(config.get("unknown"), VcxErrorKind::InvalidUrl, closure).unwrap(),
                   error::SUCCESS.code_num);

        // Fail with failed fn call
        assert_eq!(validate_optional_config_val(config.get("invalid"),
                                                VcxErrorKind::InvalidUrl,
                                                closure).unwrap_err().kind(), VcxErrorKind::InvalidUrl);
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

    #[test]
    #[cfg(feature = "general_test")]
    fn test_clear_config() {
        let _setup = SetupDefaults::init();

        let content = json!({
            "pool_name" : "pool1",
            "config_name":"config1",
            "wallet_name":"test_clear_config",
            "institution_name" : "evernym enterprise",
            "genesis_path":"/tmp/pool1.txn",
            "wallet_key":"key",
        }).to_string();

        assert_eq!(process_config_string(&content, false).unwrap(), error::SUCCESS.code_num);

        assert_eq!(get_config_value("pool_name").unwrap(), "pool1".to_string());
        assert_eq!(get_config_value("config_name").unwrap(), "config1".to_string());
        assert_eq!(get_config_value("wallet_name").unwrap(), "test_clear_config".to_string());
        assert_eq!(get_config_value("institution_name").unwrap(), "evernym enterprise".to_string());
        assert_eq!(get_config_value("genesis_path").unwrap(), "/tmp/pool1.txn".to_string());
        assert_eq!(get_config_value("wallet_key").unwrap(), "key".to_string());

        clear_config();

        // Fails after config is cleared
        assert_eq!(get_config_value("pool_name").unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
        assert_eq!(get_config_value("config_name").unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
        assert_eq!(get_config_value("wallet_name").unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
        assert_eq!(get_config_value("institution_name").unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
        assert_eq!(get_config_value("genesis_path").unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
        assert_eq!(get_config_value("wallet_key").unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_process_config_str_for_actors() {
        let _setup = SetupDefaults::init();

        let mut config = base_config();
        config["actors"] = json!(["invitee", "holder"]);

        process_config_string(&config.to_string(), true).unwrap();

        assert_eq!(vec![Actors::Invitee, Actors::Holder], get_actors());

        // passed invalid actor
        config["actors"] = json!(["wrong"]);
        assert_eq!(process_config_string(&config.to_string(), true).unwrap_err().kind(), VcxErrorKind::InvalidOption);
    }
}
