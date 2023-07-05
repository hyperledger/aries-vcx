use std::collections::HashMap;
use std::sync::RwLock;

use crate::errors::error::prelude::*;

pub static CONFIG_INSTITUTION_DID: &str = "institution_did";

// functionally not used
pub static CONFIG_ENABLE_TEST_MODE: &str = "enable_test_mode";
pub static CONFIG_WALLET_BACKUP_KEY: &str = "backup_key";
pub static CONFIG_WALLET_KEY: &str = "wallet_key";
pub static CONFIG_PROTOCOL_VERSION: &str = "protocol_version";
pub static CONFIG_TXN_AUTHOR_AGREEMENT: &str = "author_agreement";
pub static DEFAULT_PROTOCOL_VERSION: usize = 2;
pub static MAX_SUPPORTED_PROTOCOL_VERSION: usize = 2;
pub static DEFAULT_LINK_SECRET_ALIAS: &str = "main";
pub static DEFAULT_DID: &str = "2hoqvcwupRTUNkXn6ArYzs";

lazy_static! {
    static ref SETTINGS: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

pub fn enable_indy_mocks() -> VcxCoreResult<()> {
    debug!("enable_indy_mocks >>>");
    set_config_value(CONFIG_ENABLE_TEST_MODE, "true")
}

pub fn disable_indy_mocks() -> VcxCoreResult<()> {
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

pub fn get_config_value(key: &str) -> VcxCoreResult<String> {
    trace!("get_config_value >>> key: {}", key);

    SETTINGS
        .read()
        .or(Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::InvalidConfiguration,
            "Cannot read settings",
        )))?
        .get(key)
        .map(|v| v.to_string())
        .ok_or(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::InvalidConfiguration,
            format!("Cannot read \"{key}\" from settings"),
        ))
}

pub fn set_config_value(key: &str, value: &str) -> VcxCoreResult<()> {
    trace!("set_config_value >>> key: {}, value: {}", key, value);
    SETTINGS
        .write()
        .or(Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnknownError,
            "Cannot write settings",
        )))?
        .insert(key.to_string(), value.to_string());
    Ok(())
}

pub fn reset_config_values_ariesvcxcore() -> VcxCoreResult<()> {
    trace!("reset_config_values >>>");
    let mut config = SETTINGS.write()?;
    config.clear();
    Ok(())
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

pub fn get_sample_did() -> String {
    DEFAULT_DID.to_string()
}
