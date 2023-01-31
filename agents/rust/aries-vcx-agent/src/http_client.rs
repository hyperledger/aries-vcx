use aries_vcx::{
    agency_client::httpclient::post_message, errors::error::VcxResult, protocols::typestate_con::Transport,
};
use async_trait::async_trait;

pub struct HttpClient;

#[async_trait]
impl Transport for HttpClient {
    async fn send_message(&self, msg: Vec<u8>, service_endpoint: &str) -> VcxResult<()> {
        post_message(msg, service_endpoint).await?;
        Ok(())
    }
}
