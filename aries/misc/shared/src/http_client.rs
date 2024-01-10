use std::time::Duration;

use reqwest::{
    self,
    header::{CONTENT_TYPE, USER_AGENT},
    Client, Response, Url,
};

use crate::errors::http_error::{HttpError, HttpResult};

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

pub async fn post_message(body_content: Vec<u8>, url: &Url) -> HttpResult<Vec<u8>> {
    debug!("post_message >> http client sending request POST {}", &url);

    let response = send_post_request(url, body_content).await?;
    process_response(response).await
}

async fn send_post_request(url: &Url, body_content: Vec<u8>) -> HttpResult<Response> {
    HTTP_CLIENT
        .post(url.clone())
        .body(body_content)
        .header(CONTENT_TYPE, "application/ssi-agent-wire")
        .header(USER_AGENT, "reqwest")
        .send()
        .await
        .map_err(|err| HttpError::from_msg(format!("HTTP Client could not connect, err: {}", err)))
}

async fn process_response(response: Response) -> HttpResult<Vec<u8>> {
    let content_length = response.content_length();
    let response_status = response.status();
    match response.text().await {
        Ok(payload) => {
            if response_status.is_success() {
                Ok(payload.into_bytes())
            } else {
                Err(HttpError::from_msg(format!(
                    "POST failed due to non-success HTTP status: {}, response body: {}",
                    response_status, payload
                )))
            }
        }
        Err(error) => Err(HttpError::from_msg(format!(
            "POST failed because response could not be decoded as utf-8, HTTP status: {}, \
             content-length header: {:?}, error: {:?}",
            response_status, content_length, error
        ))),
    }
}
