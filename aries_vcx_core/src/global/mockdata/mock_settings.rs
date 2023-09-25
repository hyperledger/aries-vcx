use std::{collections::HashMap, sync::RwLock};

use crate::errors::error::VcxCoreResult;

static MOCKED_RETRIEVED_CREDS: &str = "mocked_retrieved_creds";

lazy_static! {
    static ref MOCK_SETTINGS: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
    static ref MOCK_SETTINGS_RESULT_BOOL: RwLock<HashMap<String, VcxCoreResult<bool>>> =
        RwLock::new(HashMap::new());
}

#[allow(dead_code)]
pub fn get_mock_creds_retrieved_for_proof_request() -> Option<String> {
    let config = MOCK_SETTINGS
        .read()
        .expect("Unable to access MOCK_SETTINGS");
    config.get(MOCKED_RETRIEVED_CREDS).map(String::from)
}
