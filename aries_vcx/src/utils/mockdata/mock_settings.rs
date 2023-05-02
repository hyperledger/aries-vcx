use std::sync::RwLock;
use std::{collections::HashMap, sync::Mutex};

use crate::errors::error::{AriesVcxError, VcxResult};

static MOCKED_GENERATED_PROOF: &str = "mocked_proof";
static MOCKED_RETRIEVED_CREDS: &str = "mocked_retrieved_creds";
static MOCKED_VALIDATE_INDY_PROOF: &str = "mocked_validate_indy_proof";

lazy_static! {
    static ref MOCK_SETTINGS: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
    static ref MOCK_SETTINGS_RESULT_BOOL: RwLock<HashMap<String, VcxResult<bool>>> = RwLock::new(HashMap::new());
    // todo: get rid of this, we no longer deal with rc return codes from vdrtools
    //      (this is leftover from times when we talked to vdrtool via FFI)
    static ref STATUS_CODE_MOCK: Mutex<StatusCodeMock> = Mutex::new(StatusCodeMock::default());
}

pub struct MockBuilder; // empty

impl MockBuilder {
    pub fn init() -> MockBuilder {
        MockBuilder {}
    }

    pub fn set_mock_generate_indy_proof(self, generated_proof: &str) -> MockBuilder {
        warn!(
            "MockBuilder::set_mock_generate_indy_proof >>> generated_proof: {}",
            generated_proof
        );
        let mut settings = MOCK_SETTINGS.write().expect("Unable to access MOCK_SETTINGS");
        settings.insert(String::from(MOCKED_GENERATED_PROOF), generated_proof.into());
        self
    }

    pub fn set_mock_creds_retrieved_for_proof_request(self, retrieve_creds: &str) -> MockBuilder {
        warn!(
            "MockBuilder::set_mock_creds_retrieved_for_proof_request >>> retrieve_creds: {}",
            retrieve_creds
        );
        let mut settings = MOCK_SETTINGS.write().expect("Unable to access MOCK_SETTINGS");
        settings.insert(String::from(MOCKED_RETRIEVED_CREDS), retrieve_creds.into());
        self
    }

    pub fn set_mock_result_for_validate_indy_proof(self, result: VcxResult<bool>) -> MockBuilder {
        warn!(
            "MockBuilder::set_mock_result_for_validate_indy_proof >>> result: {:?}",
            result
        );
        let mut settings = MOCK_SETTINGS_RESULT_BOOL
            .write()
            .expect("Unable to access MOCK_SETTINGS_RESULT_BOOL");
        settings.insert(String::from(MOCKED_VALIDATE_INDY_PROOF), result);
        self
    }

    pub fn reset_mock_settings(&self) {
        warn!("MockBuilder::reset_mock_settings >>>");
        let mut config = MOCK_SETTINGS.write().expect("Unable to access MOCK_SETTINGS");
        config.clear();
    }
}

impl Drop for MockBuilder {
    fn drop(&mut self) {
        warn!("MockBuilder::drop >>>");
        self.reset_mock_settings();
    }
}

pub fn get_mock_generate_indy_proof() -> Option<String> {
    let config = MOCK_SETTINGS.read().expect("Unable to access MOCK_SETTINGS");
    config.get(MOCKED_GENERATED_PROOF).map(String::from)
}

pub fn get_mock_creds_retrieved_for_proof_request() -> Option<String> {
    let config = MOCK_SETTINGS.read().expect("Unable to access MOCK_SETTINGS");
    config.get(MOCKED_RETRIEVED_CREDS).map(String::from)
}

pub fn get_mock_result_for_validate_indy_proof() -> Option<VcxResult<bool>> {
    let config = MOCK_SETTINGS_RESULT_BOOL
        .read()
        .expect("Unable to access MOCK_SETTINGS_RESULT_BOOL");
    config.get(MOCKED_VALIDATE_INDY_PROOF).map(|result| match result {
        Ok(val) => Ok(*val),
        Err(err) => Err(AriesVcxError::from_msg(err.kind(), err.to_string())),
    })
}

#[derive(Default)]
pub struct StatusCodeMock {
    results: Vec<u32>,
}

// todo: get rid of this, we no longer deal with rc return codes from vdrtools
//      (this is leftover from times when we talked to vdrtool via FFI)
impl StatusCodeMock {
    pub fn get_result() -> u32 {
        STATUS_CODE_MOCK
            .lock()
            .expect("Unable to access LIBINDY_MOCK")
            .results
            .pop()
            .unwrap_or_default()
    }
}
