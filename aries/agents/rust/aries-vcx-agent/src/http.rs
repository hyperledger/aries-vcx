use aries_vcx::{errors::error::VcxResult, transport::Transport};
use async_trait::async_trait;
use url::Url;

pub struct VcxHttpClient;

#[async_trait]
impl Transport for VcxHttpClient {
    async fn send_message(&self, msg: Vec<u8>, service_endpoint: Url) -> VcxResult<()> {
        shared::http_client::post_message(msg, service_endpoint).await?;
        Ok(())
    }
}
