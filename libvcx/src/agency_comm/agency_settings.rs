use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::RwLock;

use serde_json::Value;
use url::Url;

use error::{VcxError, VcxErrorKind, VcxResult};
use utils::{error, validation};

pub static CONFIG_AGENCY_ENDPOINT: &str = "agency_endpoint";
pub static CONFIG_AGENCY_DID: &str = "agency_did";
pub static CONFIG_AGENCY_VERKEY: &str = "agency_verkey";
pub static CONFIG_REMOTE_TO_SDK_DID: &str = "remote_to_sdk_did";
pub static CONFIG_REMOTE_TO_SDK_VERKEY: &str = "remote_to_sdk_verkey";
pub static CONFIG_SDK_TO_REMOTE_DID: &str = "sdk_to_remote_did";
pub static CONFIG_SDK_TO_REMOTE_VERKEY: &str = "sdk_to_remote_verkey";
static CONFIG_ENABLE_TEST_MODE: &str = "enable_test_mode";

pub static VALID_AGENCY_CONFIG_KEYS: &[&str] = &[
    CONFIG_AGENCY_ENDPOINT,
    CONFIG_AGENCY_DID,
    CONFIG_AGENCY_VERKEY,
    CONFIG_REMOTE_TO_SDK_DID,
    CONFIG_REMOTE_TO_SDK_VERKEY,
    CONFIG_SDK_TO_REMOTE_DID,
    CONFIG_SDK_TO_REMOTE_VERKEY,
    CONFIG_ENABLE_TEST_MODE
];

lazy_static! {
    static ref AGENCY_SETTINGS: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
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

pub fn set_testing_defaults_agency() -> u32 {
    trace!("set_testing_defaults_agency >>>");

    let DEFAULT_DID= "2hoqvcwupRTUNkXn6ArYzs";
    let DEFAULT_VERKEY= "FuN98eH2eZybECWkofW6A9BKJxxnTatBCopfUiNxo6ZB";
    let DEFAULT_URL= "http://127.0.0.1:8080";

    // if this fails the test should exit
    let mut agency_settings = AGENCY_SETTINGS.write().unwrap();

    agency_settings.insert(CONFIG_AGENCY_ENDPOINT.to_string(), DEFAULT_URL.to_string());
    agency_settings.insert(CONFIG_AGENCY_DID.to_string(), DEFAULT_DID.to_string());
    agency_settings.insert(CONFIG_AGENCY_VERKEY.to_string(), DEFAULT_VERKEY.to_string());
    agency_settings.insert(CONFIG_REMOTE_TO_SDK_DID.to_string(), DEFAULT_DID.to_string());
    agency_settings.insert(CONFIG_REMOTE_TO_SDK_VERKEY.to_string(), DEFAULT_VERKEY.to_string());
    agency_settings.insert(CONFIG_SDK_TO_REMOTE_DID.to_string(), DEFAULT_DID.to_string());
    agency_settings.insert(CONFIG_SDK_TO_REMOTE_VERKEY.to_string(), DEFAULT_VERKEY.to_string());

    error::SUCCESS.code_num
}

pub fn clear_config_agency() {
    trace!("clear_config_agency >>>");
    let mut config = AGENCY_SETTINGS.write().unwrap();
    config.clear();
}

pub fn validate_agency_config(config: &HashMap<String, String>) -> VcxResult<u32> {
    trace!("validate_agency_config >>> config: {:?}", config);

    // todo: Since we scope these setting to agency module, these are not really optional anymore!
    validate_optional_config_val(config.get(CONFIG_AGENCY_DID), VcxErrorKind::InvalidDid, validation::validate_did)?;
    validate_optional_config_val(config.get(CONFIG_AGENCY_VERKEY), VcxErrorKind::InvalidVerkey, validation::validate_verkey)?;

    validate_optional_config_val(config.get(CONFIG_SDK_TO_REMOTE_DID), VcxErrorKind::InvalidDid, validation::validate_did)?;
    validate_optional_config_val(config.get(CONFIG_SDK_TO_REMOTE_VERKEY), VcxErrorKind::InvalidVerkey, validation::validate_verkey)?;

    validate_optional_config_val(config.get(CONFIG_REMOTE_TO_SDK_DID), VcxErrorKind::InvalidDid, validation::validate_did)?;
    validate_optional_config_val(config.get(CONFIG_REMOTE_TO_SDK_VERKEY), VcxErrorKind::InvalidVerkey, validation::validate_verkey)?;

    validate_optional_config_val(config.get(CONFIG_AGENCY_ENDPOINT), VcxErrorKind::InvalidUrl, Url::parse)?;

    Ok(error::SUCCESS.code_num)
}


pub fn process_agency_config_string(config: &str, do_validation: bool) -> VcxResult<u32> {
    trace!("process_config_string >>> config {}", config);

    let configuration: Value = serde_json::from_str(config)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot parse config: {}", err)))?;

    if let Value::Object(ref map) = configuration {
        for (key, value) in map {
            if !VALID_AGENCY_CONFIG_KEYS.contains(&key.as_ref()) {
                warn!("Agency settings do not recognize setting key {}. Will be ignored.", key);
            } else {
                match value {
                    Value::String(value_) => set_config_value(key, &value_),
                    Value::Bool(value_) => set_config_value(key, &json!(value_).to_string()),
                    _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidJson,
                                                       format!("Invalid agency config value for key {}", key))),
                }
            }
        }
    }

    if do_validation {
        let setting = AGENCY_SETTINGS.read()
            .or(Err(VcxError::from(VcxErrorKind::InvalidConfiguration)))?;
        validate_agency_config(&setting.borrow())
    } else {
        Ok(error::SUCCESS.code_num)
    }
}


pub fn get_config_value(key: &str) -> VcxResult<String> {
    trace!("get_config_value >>> key: {}", key);

    AGENCY_SETTINGS
        .read()
        .or(Err(VcxError::from_msg(VcxErrorKind::InvalidConfiguration, "Cannot read AGENCY_SETTINGS")))?
        .get(key)
        .map(|v| v.to_string())
        .ok_or(VcxError::from_msg(VcxErrorKind::InvalidConfiguration, format!("Cannot read \"{}\" from AGENCY_SETTINGS", key)))
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


