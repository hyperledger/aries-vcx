use std::{env, time::Duration};

use reqwest::{
    self,
    header::{CONTENT_TYPE, USER_AGENT},
    Client,
};

use crate::{
    errors::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult},
    testing::{
        mocking,
        mocking::{AgencyMock, AgencyMockDecrypted, HttpClientMockResponse},
    },
};

lazy_static! {
    static ref HTTP_CLIENT: Client = {
        match reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(50))
            .pool_idle_timeout(Some(Duration::from_secs(4)))
            .build()
        {
            Ok(client) => client,
            Err(e) => panic!("Building reqwest client failed: {:?}", e),
        }
    };
}

pub async fn post_message(body_content: Vec<u8>, url: &str) -> AgencyClientResult<Vec<u8>> {
    if mocking::agency_mocks_enabled() {
        if HttpClientMockResponse::has_response() {
            warn!("post_message >> mocking response for POST {}", url);
            return HttpClientMockResponse::get_response();
        }
        if AgencyMockDecrypted::has_decrypted_mock_responses() {
            warn!("post_message >> will use mocked decrypted response for POST {}", url);
            return Ok(vec![]);
        }
        let mocked_response = AgencyMock::get_response();
        warn!(
            "post_message >> mocking response of length {} for POST {}",
            mocked_response.len(),
            url
        );
        return Ok(mocked_response);
    }

    // Setting SSL Certs location. This is needed on android
    // platform. Or openssl will fail to verify the certs
    if cfg!(target_os = "android") {
        info!("::Android code");
        set_ssl_cert_location();
    }

    debug!("post_message >> http client sending request POST {}", url);

    let response = HTTP_CLIENT
        .post(url)
        .body(body_content)
        .header(CONTENT_TYPE, "application/ssi-agent-wire")
        .header(USER_AGENT, "reqwest")
        .send()
        .await
        .map_err(|err| {
            AgencyClientError::from_msg(
                AgencyClientErrorKind::PostMessageFailed,
                format!("HTTP Client could not connect with {}, err: {}", url, err),
            )
        })?;

    let content_length = response.content_length();
    let response_status = response.status();
    match response.text().await {
        Ok(payload) => {
            if response_status.is_success() {
                Ok(payload.into_bytes())
            } else {
                Err(AgencyClientError::from_msg(
                    AgencyClientErrorKind::PostMessageFailed,
                    format!(
                        "POST {} failed due to non-success HTTP status: {}, response body: {}",
                        url, response_status, payload
                    ),
                ))
            }
        }
        Err(error) => Err(AgencyClientError::from_msg(
            AgencyClientErrorKind::PostMessageFailed,
            format!(
                "POST {} failed because response could not be decoded as utf-8, HTTP status: {}, content-length \
                 header: {:?}, error: {:?}",
                url, response_status, content_length, error
            ),
        )),
    }
}

fn set_ssl_cert_location() {
    let ssl_cert_file = "SSL_CERT_FILE";

    let external_storage =
        env::var("EXTERNAL_STORAGE").expect("the evironment variable 'EXTERNAL_STORAGE' has not been set correctly");

    // TODO: CHANGE ME, HARDCODING FOR TESTING ONLY
    env::set_var(ssl_cert_file, external_storage + "/cacert.pem");

    match env::var(ssl_cert_file) {
        Ok(val) => info!("{}:: {:?}", ssl_cert_file, val),
        Err(e) => error!(
            "couldn't find var in env {}:: {}. \
                          This needs to be set on Android to make https calls.\n\
                          See https://github.com/seanmonstar/reqwest/issues/70 for more info",
            ssl_cert_file, e
        ),
    }
    info!("::SSL_CERT_FILE has been set");
}
