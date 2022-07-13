use std::sync::Mutex;

use crate::agency_settings;
use crate::agency_settings::{disable_agency_test_mode, enable_agency_test_mode, get_config_enable_test_mode};
use crate::error::AgencyClientResult;

lazy_static! {
    static ref AGENCY_MOCK: Mutex<AgencyMock> = Mutex::new(AgencyMock::default());
    static ref AGENCY_MOCK_DECRYPTED_RESPONSES: Mutex<AgencyMockDecrypted> = Mutex::new(AgencyMockDecrypted::default());
    static ref AGENCY_MOCK_DECRYPTED_MESSAGES: Mutex<AgencyMockDecryptedMessages> = Mutex::new(AgencyMockDecryptedMessages::default());
    static ref HTTPCLIENT_MOCK_RESPONSES: Mutex<HttpClientMockResponse> = Mutex::new(HttpClientMockResponse::default());
}

#[derive(Default)]
pub struct HttpClientMockResponse {
    responses: Vec<AgencyClientResult<Vec<u8>>>,
}

impl HttpClientMockResponse {
    pub fn set_next_response(response: AgencyClientResult<Vec<u8>>) {
        if agency_mocks_enabled() {
            HTTPCLIENT_MOCK_RESPONSES.lock().unwrap().responses.push(response);
        }
    }

    pub fn has_response() -> bool {
        HTTPCLIENT_MOCK_RESPONSES.lock().unwrap().responses.len() > 0
    }

    pub fn get_response() -> AgencyClientResult<Vec<u8>> {
        HTTPCLIENT_MOCK_RESPONSES.lock().unwrap().responses.pop().unwrap()
    }
}

#[derive(Default)]
pub struct AgencyMockDecryptedMessages {
    messages: Vec<String>,
}

#[derive(Default)]
pub struct AgencyMock {
    responses: Vec<Vec<u8>>,
}

#[derive(Default)]
pub struct AgencyMockDecrypted {
    responses: Vec<String>,
}

impl AgencyMock {
    pub fn set_next_response(body: Vec<u8>) {
        if agency_mocks_enabled() {
            AGENCY_MOCK.lock().unwrap().responses.push(body);
        }
    }

    pub fn get_response() -> Vec<u8> {
        AGENCY_MOCK.lock().unwrap().responses.pop().unwrap_or_default()
    }
}

impl AgencyMockDecrypted {
    pub fn set_next_decrypted_response(body: &str) {
        if agency_mocks_enabled() {
            trace!("Mocks enabled, setting next decrypted response");
            AGENCY_MOCK_DECRYPTED_RESPONSES.lock().unwrap().responses.push(body.into());
        } else {
            warn!("Attempting to set mocked decrypted response when mocks are not enabled!");
        }
    }

    pub fn get_next_decrypted_response() -> String {
        if Self::has_decrypted_mock_responses() {
            trace!("Mocks enabled, getting next decrypted response");
            AGENCY_MOCK_DECRYPTED_RESPONSES.lock().unwrap().responses.pop().unwrap()
        } else {
            debug!("Attempting to obtain decrypted response when none were set, but decrypted messages available - returning empty response...");
            String::new()
        }
    }

    pub fn has_decrypted_mock_responses() -> bool {
        AGENCY_MOCK_DECRYPTED_RESPONSES.lock().unwrap().responses.len() > 0
    }

    pub fn set_next_decrypted_message(message: &str) {
        if agency_mocks_enabled() {
            trace!("Mocks enabled, getting next decrypted message");
            AGENCY_MOCK_DECRYPTED_MESSAGES.lock().unwrap().messages.push(message.into());
        } else {
            warn!("Attempting to set mocked decrypted message when mocks are not enabled!");
        }
    }

    pub fn get_next_decrypted_message() -> String {
        AGENCY_MOCK_DECRYPTED_MESSAGES.lock().unwrap().messages.pop().unwrap()
    }

    pub fn has_decrypted_mock_messages() -> bool {
        AGENCY_MOCK_DECRYPTED_MESSAGES.lock().unwrap().messages.len() > 0
    }

    pub fn clear_mocks() {
        AGENCY_MOCK_DECRYPTED_MESSAGES.lock().unwrap().messages.clear();
        AGENCY_MOCK_DECRYPTED_RESPONSES.lock().unwrap().responses.clear();
    }
}

pub fn agency_mocks_enabled() -> bool {
    match get_config_enable_test_mode().ok() {
        None => false,
        Some(value) => value == "true" || value == "agency"
    }
}

pub fn agency_decrypted_mocks_enabled() -> bool {
    match get_config_enable_test_mode().ok() {
        None => false,
        Some(value) => value == "true"
    }
}

pub fn enable_agency_mocks() {
    enable_agency_test_mode()
}

pub fn disable_agency_mocks() {
    disable_agency_test_mode();
}
