use std::env;
use std::io::Read;
use std::sync::Mutex;

use reqwest;
use reqwest::header::CONTENT_TYPE;

use error::prelude::*;
use settings;

lazy_static! {
    static ref AGENCY_MOCK: Mutex<AgencyMock> = Mutex::new(AgencyMock::default());
    static ref AGENCY_MOCK_DECRYPTED_RESPONSES: Mutex<AgencyMockDecrypted> = Mutex::new(AgencyMockDecrypted::default());
    static ref AGENCY_MOCK_DECRYPTED_MESSAGES: Mutex<AgencyMockDecryptedMessages> = Mutex::new(AgencyMockDecryptedMessages::default());
    static ref HTTPCLIENT_MOCK_RESPONSES: Mutex<HttpClientMockResponse> = Mutex::new(HttpClientMockResponse::default());
}

#[derive(Default)]
pub struct AgencyMock {
    responses: Vec<Vec<u8>>
}

#[derive(Default)]
pub struct AgencyMockDecrypted {
    responses: Vec<String>
}

#[derive(Default)]
pub struct HttpClientMockResponse {
    responses: Vec<VcxResult<Vec<u8>>>
}

#[derive(Default)]
pub struct AgencyMockDecryptedMessages {
    messages: Vec<String>
}

impl AgencyMock {
    pub fn set_next_response(body: Vec<u8>) {
        if settings::agency_mocks_enabled() {
            AGENCY_MOCK.lock().unwrap().responses.push(body);
        }
    }

    pub fn get_response() -> Vec<u8> {
        AGENCY_MOCK.lock().unwrap().responses.pop().unwrap_or_default()
    }
}

impl HttpClientMockResponse {
    pub fn set_next_response(response: VcxResult<Vec<u8>>) {
        if settings::agency_mocks_enabled() {
            HTTPCLIENT_MOCK_RESPONSES.lock().unwrap().responses.push(response);
        }
    }

    pub fn has_response() -> bool {
        HTTPCLIENT_MOCK_RESPONSES.lock().unwrap().responses.len() > 0
    }

    pub fn get_response() -> VcxResult<Vec<u8>> {
        HTTPCLIENT_MOCK_RESPONSES.lock().unwrap().responses.pop().unwrap()
    }
}

impl AgencyMockDecrypted {
    pub fn set_next_decrypted_response(body: &str) {
        if settings::agency_mocks_enabled() {
            AGENCY_MOCK_DECRYPTED_RESPONSES.lock().unwrap().responses.push(body.into());
        } else {
            warn!("Attempting to set mocked decrypted response when mocks are not enabled!");
        }
    }

    pub fn get_next_decrypted_response() -> String {
        if !Self::has_decrypted_mock_responses() && Self::has_decrypted_mock_messages() {
            debug!("Attempting to obtain decrypted response when none were set, but decrypted messages available - returning empty response...");
            String::new()
        } else {
            AGENCY_MOCK_DECRYPTED_RESPONSES.lock().unwrap().responses.pop().unwrap()
        }
    }

    pub fn has_decrypted_mock_responses() -> bool {
        AGENCY_MOCK_DECRYPTED_RESPONSES.lock().unwrap().responses.len() > 0
    }

    pub fn set_next_decrypted_message(message: &str) {
        if settings::agency_mocks_enabled() {
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

//Todo: change this RC to a u32
pub fn post_u8(body_content: &Vec<u8>) -> VcxResult<Vec<u8>> {
    let endpoint = format!("{}/agency/msg", settings::get_config_value(settings::CONFIG_AGENCY_ENDPOINT)?);
    post_message(body_content, &endpoint)
}

pub fn post_message(body_content: &Vec<u8>, url: &str) -> VcxResult<Vec<u8>> {
    if settings::agency_mocks_enabled() {
        if HttpClientMockResponse::has_response() {
            warn!("HttpClient has mocked response");
            return HttpClientMockResponse::get_response();
        }
        if AgencyMockDecrypted::has_decrypted_mock_responses() {
            warn!("Agency requests returns empty response, decrypted mock response is available");
            return Ok(vec!());
        }
        let mocked_response = AgencyMock::get_response();
        debug!("Agency returns mocked response of length {}", mocked_response.len());
        return Ok(mocked_response);
    }

    //Setting SSL Certs location. This is needed on android platform. Or openssl will fail to verify the certs
    if cfg!(target_os = "android") {
        info!("::Android code");
        set_ssl_cert_location();
    }
    let client = reqwest::ClientBuilder::new().timeout(::utils::timeout::TimeoutUtils::long_timeout()).build().map_err(|err| {
        error!("error: {}", err);
        VcxError::from_msg(VcxErrorKind::PostMessageFailed, format!("Building reqwest client failed: {:?}", err))
    })?;
    debug!("Posting encrypted bundle to: \"{}\"", url);

    let mut response =
        client.post(url)
            .body(body_content.to_owned())
            .header(CONTENT_TYPE, "application/ssi-agent-wire")
            .send()
            .map_err(|err| {
                error!("error: {}", err);
                VcxError::from_msg(VcxErrorKind::PostMessageFailed, format!("Could not connect {:?}", err))
            })?;

    trace!("Response Header: {:?}", response);
    if !response.status().is_success() {
        let mut content = String::new();
        match response.read_to_string(&mut content) {
            Ok(_) => info!("Request failed: {}", content),
            Err(_) => info!("could not read response"),
        };
        return Err(VcxError::from_msg(VcxErrorKind::PostMessageFailed, format!("POST failed with: {}", content)));
    }

    let mut content = Vec::new();
    response.read_to_end(&mut content)
        .or(Err(VcxError::from_msg(VcxErrorKind::PostMessageFailed, "could not read response")))?;

    Ok(content)
}

fn set_ssl_cert_location() {
    let ssl_cert_file = "SSL_CERT_FILE";
    env::set_var(ssl_cert_file, env::var("EXTERNAL_STORAGE").unwrap() + "/cacert.pem"); //TODO: CHANGE ME, HARDCODING FOR TESTING ONLY
    match env::var(ssl_cert_file) {
        Ok(val) => info!("{}:: {:?}", ssl_cert_file, val),
        Err(e) => error!("couldn't find var in env {}:: {}. This needs to be set on Android to make https calls.\n See https://github.com/seanmonstar/reqwest/issues/70 for more info",
                         ssl_cert_file, e),
    }
    info!("::SSL_CERT_FILE has been set");
}
