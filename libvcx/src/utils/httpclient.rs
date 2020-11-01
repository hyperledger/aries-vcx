use std::env;
use std::io::Read;
use std::sync::Mutex;

use reqwest;
use reqwest::header::CONTENT_TYPE;

use agency_comm::agency_settings;
use agency_comm::mocking::{AgencyMock, AgencyMockDecrypted};
use error::prelude::*;
use settings;

lazy_static! {
    static ref HTTPCLIENT_MOCK_RESPONSES: Mutex<HttpClientMockResponse> = Mutex::new(HttpClientMockResponse::default());
}

#[derive(Default)]
pub struct HttpClientMockResponse {
    responses: Vec<VcxResult<Vec<u8>>>
}

impl HttpClientMockResponse {
    pub fn set_next_response(response: VcxResult<Vec<u8>>) {
        if agency_settings::agency_mocks_enabled() {
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

pub fn post_message(body_content: &Vec<u8>, url: &str) -> VcxResult<Vec<u8>> {
    // todo: this function should be general, not knowing that agency exists -> move agency mocks to agency module
    if agency_settings::agency_mocks_enabled() {
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
