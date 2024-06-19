use aries_vcx::{errors::error::VcxResult, transport::Transport};
use async_trait::async_trait;
use url::Url;

pub struct VcxHttpClient;

#[async_trait]
impl Transport for VcxHttpClient {
    async fn send_message(&self, msg: Vec<u8>, _service_endpoint: &Url) -> VcxResult<()> {
        // TODO - 1288 - REMOVE THIS.
        let service_endpoint = &"http://localhost:9031".parse().unwrap();
        shared::http_client::post_message(msg, service_endpoint).await?;
        Ok(())
    }
}
