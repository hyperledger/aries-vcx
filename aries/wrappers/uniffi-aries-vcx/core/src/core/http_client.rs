use aries_vcx::{errors::error::VcxResult, transport::Transport};
use async_trait::async_trait;
use shared::http_client::post_message;
use url::Url;
pub struct HttpClient;

#[async_trait]
impl Transport for HttpClient {
    async fn send_message(&self, msg: Vec<u8>, service_endpoint: &Url) -> VcxResult<()> {
        post_message(msg, service_endpoint).await?;
        Ok(())
    }
}
