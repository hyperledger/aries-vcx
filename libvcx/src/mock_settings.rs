use std::collections::HashMap;
use std::sync::RwLock;

pub static MOCKED_GENERATED_PROOF: &str = "mocked_proof";
pub static MOCKED_RETRIEVED_CREDS: &str = "mocked_retrieved_creds";

lazy_static! {
    static ref MOCK_SETTINGS: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

pub fn set_mock_generate_indy_proof(generated_proof: &str) {
    let mut settings = MOCK_SETTINGS.write().unwrap();
    settings.insert(String::from(MOCKED_GENERATED_PROOF), generated_proof.into());
}

pub fn get_mock_generate_indy_proof() -> Option<String> {
    let config = MOCK_SETTINGS.read().unwrap();
    config.get(MOCKED_GENERATED_PROOF).map(|s| String::from(s))
}

pub fn set_mock_creds_retrieved_for_proof_request(retrieve_creds: &str) {
    let mut settings = MOCK_SETTINGS.write().unwrap();
    settings.insert(String::from(MOCKED_RETRIEVED_CREDS), retrieve_creds.into());
}

pub fn get_mock_creds_retrieved_for_proof_request() -> Option<String> {
    let config = MOCK_SETTINGS.read().unwrap();
    config.get(MOCKED_RETRIEVED_CREDS).map(|s| String::from(s))
}

pub fn reset_mock_settings() {
    let mut config = MOCK_SETTINGS.write().unwrap();
    config.clear();
}
