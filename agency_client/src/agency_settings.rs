use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::RwLock;

use serde_json::Value;
use url::Url;
use crate::agency_settings;

use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::utils::{error_utils, validation};

pub const CONFIG_AGENCY_ENDPOINT: &str = "agency_endpoint";
pub const CONFIG_AGENCY_DID: &str = "agency_did";
pub const CONFIG_AGENCY_VERKEY: &str = "agency_verkey";
pub const CONFIG_REMOTE_TO_SDK_DID: &str = "remote_to_sdk_did";
pub const CONFIG_REMOTE_TO_SDK_VERKEY: &str = "remote_to_sdk_verkey";
pub const CONFIG_SDK_TO_REMOTE_DID: &str = "sdk_to_remote_did";
pub const CONFIG_SDK_TO_REMOTE_VERKEY: &str = "sdk_to_remote_verkey";
pub const CONFIG_ENABLE_TEST_MODE: &str = "enable_test_mode";
pub const CONFIG_WALLET_HANDLE: &str = "wallet_handle";

pub static VALID_AGENCY_CONFIG_KEYS: &[&str] = &[
    CONFIG_AGENCY_ENDPOINT,
    CONFIG_AGENCY_DID,
    CONFIG_AGENCY_VERKEY,
    CONFIG_REMOTE_TO_SDK_DID,
    CONFIG_REMOTE_TO_SDK_VERKEY,
    CONFIG_SDK_TO_REMOTE_DID,
    CONFIG_SDK_TO_REMOTE_VERKEY,
    CONFIG_ENABLE_TEST_MODE,
    CONFIG_WALLET_HANDLE,
];

lazy_static! {
    static ref AGENCY_SETTINGS: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}


pub fn validate_optional_config_val<F, S, E>(val: Option<&String>, err: AgencyClientErrorKind, closure: F) -> AgencyClientResult<u32>
    where F: Fn(&str) -> Result<S, E> {
    if val.is_none() { return Ok(error_utils::SUCCESS.code_num); }

    closure(val.as_ref().ok_or(AgencyClientError::from(AgencyClientErrorKind::InvalidConfiguration))?)
        .or(Err(AgencyClientError::from(err)))?;

    Ok(error_utils::SUCCESS.code_num)
}

pub fn validate_mandotory_config_val<F, S, E>(val: &str, err: AgencyClientErrorKind, closure: F) -> AgencyClientResult<u32>
    where F: Fn(&str) -> Result<S, E> {
    closure(val)
        .or(Err(AgencyClientError::from(err)))?;

    Ok(error_utils::SUCCESS.code_num)
}

pub fn set_testing_defaults_agency() -> u32 {
    trace!("set_testing_defaults_agency >>>");

    let default_did = "VsKV7grR1BUE29mG2Fm2kX";
    let default_verkey = "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR";
    let default_url = "http://127.0.0.1:8080";

    // if this fails the test should exit
    let mut agency_settings = AGENCY_SETTINGS.write().unwrap();

    agency_settings.insert(CONFIG_AGENCY_ENDPOINT.to_string(), default_url.to_string());
    agency_settings.insert(CONFIG_AGENCY_DID.to_string(), default_did.to_string());
    agency_settings.insert(CONFIG_AGENCY_VERKEY.to_string(), default_verkey.to_string());
    agency_settings.insert(CONFIG_REMOTE_TO_SDK_DID.to_string(), default_did.to_string());
    agency_settings.insert(CONFIG_REMOTE_TO_SDK_VERKEY.to_string(), default_verkey.to_string());
    agency_settings.insert(CONFIG_SDK_TO_REMOTE_DID.to_string(), default_did.to_string());
    agency_settings.insert(CONFIG_SDK_TO_REMOTE_VERKEY.to_string(), default_verkey.to_string());

    error_utils::SUCCESS.code_num
}

pub fn clear_config_agency() {
    trace!("clear_config_agency >>>");
    let mut config = AGENCY_SETTINGS.write().unwrap();
    config.clear();
}

pub fn validate_agency_config(config: &HashMap<String, String>) -> AgencyClientResult<u32> {
    trace!("validate_agency_config >>> config: {:?}", config);

    // todo: Since we scope these setting to agency module, these are not really optional anymore!
    validate_optional_config_val(config.get(CONFIG_AGENCY_DID), AgencyClientErrorKind::InvalidDid, validation::validate_did)?;
    validate_optional_config_val(config.get(CONFIG_AGENCY_VERKEY), AgencyClientErrorKind::InvalidVerkey, validation::validate_verkey)?;

    validate_optional_config_val(config.get(CONFIG_SDK_TO_REMOTE_DID), AgencyClientErrorKind::InvalidDid, validation::validate_did)?;
    validate_optional_config_val(config.get(CONFIG_SDK_TO_REMOTE_VERKEY), AgencyClientErrorKind::InvalidVerkey, validation::validate_verkey)?;

    validate_optional_config_val(config.get(CONFIG_REMOTE_TO_SDK_DID), AgencyClientErrorKind::InvalidDid, validation::validate_did)?;
    validate_optional_config_val(config.get(CONFIG_REMOTE_TO_SDK_VERKEY), AgencyClientErrorKind::InvalidVerkey, validation::validate_verkey)?;

    validate_optional_config_val(config.get(CONFIG_AGENCY_ENDPOINT), AgencyClientErrorKind::InvalidUrl, Url::parse)?;

    Ok(error_utils::SUCCESS.code_num)
}


pub fn process_agency_config_string(config: &str, validate: bool) -> AgencyClientResult<u32> {
    trace!("process_config_string >>> config {}", config);

    let configuration: Value = serde_json::from_str(config)
        .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::InvalidJson, format!("Cannot parse config: {}", err)))?;

    if let Value::Object(ref map) = configuration {
        for (key, value) in map {
            if !VALID_AGENCY_CONFIG_KEYS.contains(&key.as_ref()) {
                warn!("Agency settings do not recognize setting key {}. Will be ignored.", key);
            } else {
                match value {
                    Value::String(value_) => set_config_value(key, &value_),
                    Value::Bool(value_) => set_config_value(key, &json!(value_).to_string()),
                    _ => return Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidJson,
                                                                format!("Invalid agency config value for key {}", key))),
                }
            }
        }
    }

    if validate {
        let setting = AGENCY_SETTINGS.read()
            .or(Err(AgencyClientError::from(AgencyClientErrorKind::InvalidConfiguration)))?;
        validate_agency_config(&setting.borrow())
    } else {
        Ok(error_utils::SUCCESS.code_num)
    }
}

pub fn get_config_enable_test_mode() -> AgencyClientResult<String> {
    get_config_value(agency_settings::CONFIG_ENABLE_TEST_MODE)
}

pub fn enable_agency_test_mode() {
    agency_settings::set_config_value(agency_settings::CONFIG_ENABLE_TEST_MODE, "true");
}

pub fn disable_agency_test_mode() {
    agency_settings::set_config_value(agency_settings::CONFIG_ENABLE_TEST_MODE, "false");
}

pub fn set_config_value(key: &str, value: &str) {
    trace!("set_config_value >>> key: {}, value: {}", key, value);
    if !VALID_AGENCY_CONFIG_KEYS.contains(&key) {
        warn!("Agency settings do not recognize setting key {}. Will be ignored.", key);
    } else {
        AGENCY_SETTINGS
            .write().unwrap()
            .insert(key.to_string(), value.to_string());
    }
}

pub fn get_config_value(key: &str) -> AgencyClientResult<String> {
    trace!("get_config_value >>> key: {}", key);

    AGENCY_SETTINGS
        .read()
        .or(Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidConfiguration, "Cannot read AGENCY_SETTINGS")))?
        .get(key)
        .map(|v| v.to_string())
        .ok_or(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidConfiguration, format!("Cannot read \"{}\" from AGENCY_SETTINGS", key)))
}