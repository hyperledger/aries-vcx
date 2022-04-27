use std::sync::Mutex;

use crate::settings;

lazy_static! {
    static ref POOL_MOCK_RESPONSES: Mutex<PoolMocks> = Mutex::new(PoolMocks::default());
}

#[derive(Default)]
pub struct PoolMocks {
    responses: Vec<String>,
}

impl PoolMocks {
    pub fn set_next_pool_response(body: &str) {
        if pool_mocks_enabled() {
            trace!("Mocks enabled, setting next pool response");
            POOL_MOCK_RESPONSES.lock().unwrap().responses.push(body.into());
        } else {
            warn!("Attempting to set mocked pool response when mocks are not enabled!");
        }
    }

    pub fn get_next_pool_response() -> String {
        if Self::has_pool_mock_responses() {
            trace!("Mocks enabled, getting next pool response");
            POOL_MOCK_RESPONSES.lock().unwrap().responses.pop().unwrap()
        } else {
            debug!("Attempting to obtain pool response when none were set, but pool messages available - returning empty response...");
            String::new()
        }
    }

    pub fn has_pool_mock_responses() -> bool {
        POOL_MOCK_RESPONSES.lock().unwrap().responses.len() > 0
    }

    pub fn clear_mocks() {
        POOL_MOCK_RESPONSES.lock().unwrap().responses.clear();
    }
}

pub fn pool_mocks_enabled() -> bool {
    match settings::get_config_value(settings::CONFIG_ENABLE_TEST_MODE).ok() {
        None => false,
        Some(value) => value == "pool"
    }
}

pub fn enable_pool_mocks() {
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "pool");
}

pub fn disable_pool_mocks() {
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "pool");
}
