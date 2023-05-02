use std::sync::RwLock;
use std::{collections::HashMap, sync::Mutex};

use crate::errors::error::VcxCoreResult;

static MOCKED_RETRIEVED_CREDS: &str = "mocked_retrieved_creds";

lazy_static! {
    static ref MOCK_SETTINGS: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
    static ref MOCK_SETTINGS_RESULT_BOOL: RwLock<HashMap<String, VcxCoreResult<bool>>> = RwLock::new(HashMap::new());
    static ref STATUS_CODE_MOCK: Mutex<StatusCodeMock> = Mutex::new(StatusCodeMock::default());
}

pub fn get_mock_creds_retrieved_for_proof_request() -> Option<String> {
    let config = MOCK_SETTINGS.read().expect("Unable to access MOCK_SETTINGS");
    config.get(MOCKED_RETRIEVED_CREDS).map(String::from)
}

#[derive(Default)]
pub struct StatusCodeMock {
    results: Vec<u32>,
}

// todo: get rid of this, we no longer deal with rc return codes from vdrtools
//      (this is leftover from times when we talked to vdrtool via FFI)
impl StatusCodeMock {
    pub fn set_next_result(rc: u32) {
        STATUS_CODE_MOCK
            .lock()
            .expect("Unabled to access LIBINDY_MOCK")
            .results
            .push(rc);
    }

    pub fn get_result() -> u32 {
        STATUS_CODE_MOCK
            .lock()
            .expect("Unable to access LIBINDY_MOCK")
            .results
            .pop()
            .unwrap_or_default()
    }
}
