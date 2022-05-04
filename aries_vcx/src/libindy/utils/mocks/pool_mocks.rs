use std::sync::Mutex;

use crate::libindy::utils::mocks::ENABLED_MOCKS;

lazy_static! {
    static ref POOL_MOCK_RESPONSES: Mutex<PoolMocks> = Mutex::new(PoolMocks::default());
}

pub const CONFIG_POOL_MOCKS: &str = "pool_mocks";

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
    ENABLED_MOCKS
        .read().unwrap()
        .contains(CONFIG_POOL_MOCKS)
}

pub fn enable_pool_mocks() {
    ENABLED_MOCKS
        .write().unwrap()
        .insert(CONFIG_POOL_MOCKS.to_string());
}

pub fn disable_pool_mocks() {
    ENABLED_MOCKS
        .write().unwrap()
        .remove(CONFIG_POOL_MOCKS);
}
