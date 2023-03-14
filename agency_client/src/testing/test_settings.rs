use std::{collections::HashMap, sync::RwLock};

use crate::errors::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};

const CONFIG_AGENCY_TEST_MODE: &str = "enable_test_mode";

static VALID_AGENCY_CONFIG_KEYS: &[&str] = &[CONFIG_AGENCY_TEST_MODE];

lazy_static! {
    static ref AGENCY_SETTINGS: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

pub fn get_config_agency_test_mode() -> AgencyClientResult<String> {
    _get_config_value(CONFIG_AGENCY_TEST_MODE)
}

pub fn enable_agency_test_mode() {
    _set_test_config(CONFIG_AGENCY_TEST_MODE, "true");
}

pub fn disable_agency_test_mode() {
    _set_test_config(CONFIG_AGENCY_TEST_MODE, "false");
}

fn _set_test_config(key: &str, value: &str) {
    trace!("set_config_value >>> key: {}, value: {}", key, value);
    if !VALID_AGENCY_CONFIG_KEYS.contains(&key) {
        warn!("Agency settings do not recognize setting key {}. Will be ignored.", key);
    } else {
        AGENCY_SETTINGS
            .write()
            .expect("Could not write to AGENCY_SETTINGS")
            .insert(key.to_string(), value.to_string());
    }
}

fn _get_config_value(key: &str) -> AgencyClientResult<String> {
    trace!("get_config_value >>> key: {}", key);

    AGENCY_SETTINGS
        .read()
        .or(Err(AgencyClientError::from_msg(
            AgencyClientErrorKind::InvalidConfiguration,
            "Cannot read AGENCY_SETTINGS",
        )))?
        .get(key)
        .map(|v| v.to_string())
        .ok_or(AgencyClientError::from_msg(
            AgencyClientErrorKind::InvalidConfiguration,
            format!("Cannot read \"{}\" from AGENCY_SETTINGS", key),
        ))
}
