use std::sync::Mutex;

use agency_comm::agency_settings;

lazy_static! {
    static ref AGENCY_MOCK: Mutex<AgencyMock> = Mutex::new(AgencyMock::default());
    static ref AGENCY_MOCK_DECRYPTED_RESPONSES: Mutex<AgencyMockDecrypted> = Mutex::new(AgencyMockDecrypted::default());
    static ref AGENCY_MOCK_DECRYPTED_MESSAGES: Mutex<AgencyMockDecryptedMessages> = Mutex::new(AgencyMockDecryptedMessages::default());
}

#[derive(Default)]
pub struct AgencyMockDecryptedMessages {
    messages: Vec<String>
}

#[derive(Default)]
pub struct AgencyMock {
    responses: Vec<Vec<u8>>
}

#[derive(Default)]
pub struct AgencyMockDecrypted {
    responses: Vec<String>
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
            AGENCY_MOCK_DECRYPTED_RESPONSES.lock().unwrap().responses.push(body.into());
        } else {
            warn!("Attempting to set mocked decrypted response when mocks are not enabled!");
        }
    }

    pub fn get_next_decrypted_response() -> String {
        if Self::has_decrypted_mock_responses() {
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
    match agency_settings::get_config_value(agency_settings::CONFIG_ENABLE_TEST_MODE).ok() {
        None => false,
        Some(value) => value == "true" || value == "agency"
    }
}

pub fn agency_decrypted_mocks_enabled() -> bool {
    match agency_settings::get_config_value(agency_settings::CONFIG_ENABLE_TEST_MODE).ok() {
        None => false,
        Some(value) => value == "true"
    }
}

