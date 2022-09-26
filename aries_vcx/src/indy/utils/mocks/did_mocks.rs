use std::sync::Mutex;

use crate::indy::utils::mocks::ENABLED_MOCKS;

lazy_static! {
    static ref DID_MOCK_RESPONSES: Mutex<DidMocks> = Mutex::new(DidMocks::default());
}

pub const CONFIG_DID_MOCKS: &str = "did_mocks";

#[derive(Default)]
pub struct DidMocks {
    responses: Vec<String>,
}

impl DidMocks {
    pub fn set_next_did_response(body: &str) {
        if did_mocks_enabled() {
            trace!("Mocks enabled, setting next did response");
            DID_MOCK_RESPONSES.lock().unwrap().responses.push(body.into());
        } else {
            warn!("Attempting to set mocked did response when mocks are not enabled!");
        }
    }

    pub fn get_next_did_response() -> String {
        if Self::has_did_mock_responses() {
            trace!("Mocks enabled, getting next did response");
            DID_MOCK_RESPONSES.lock().unwrap().responses.pop().unwrap()
        } else {
            debug!("Attempting to obtain did response when none were set, but did messages available - returning empty response...");
            String::new()
        }
    }

    pub fn has_did_mock_responses() -> bool {
        !DID_MOCK_RESPONSES.lock().unwrap().responses.is_empty()
    }

    pub fn clear_mocks() {
        DID_MOCK_RESPONSES.lock().unwrap().responses.clear();
    }
}

pub fn did_mocks_enabled() -> bool {
    ENABLED_MOCKS.read().unwrap().contains(CONFIG_DID_MOCKS)
}

pub fn enable_did_mocks() {
    ENABLED_MOCKS.write().unwrap().insert(CONFIG_DID_MOCKS.to_string());
}

pub fn disable_did_mocks() {
    ENABLED_MOCKS.write().unwrap().remove(CONFIG_DID_MOCKS);
}
