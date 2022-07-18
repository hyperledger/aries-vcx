use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::RwLock;

use serde_json::Value;
use url::Url;

use crate::agency_settings;
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::utils::{error_utils, validation};

pub const CONFIG_ENABLE_TEST_MODE: &str = "enable_test_mode";

pub static VALID_AGENCY_CONFIG_KEYS: &[&str] = &[
    CONFIG_ENABLE_TEST_MODE,
];

lazy_static! {
    static ref AGENCY_SETTINGS: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

pub fn get_config_enable_test_mode() -> AgencyClientResult<String> {
    get_config_value(agency_settings::CONFIG_ENABLE_TEST_MODE)
}

pub fn enable_agency_test_mode() {
    agency_settings::set_test_config(agency_settings::CONFIG_ENABLE_TEST_MODE, "true");
}

pub fn disable_agency_test_mode() {
    agency_settings::set_test_config(agency_settings::CONFIG_ENABLE_TEST_MODE, "false");
}

pub fn set_test_config(key: &str, value: &str) {
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