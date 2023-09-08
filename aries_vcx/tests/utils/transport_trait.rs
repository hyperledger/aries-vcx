use aries_vcx::{errors::error::VcxResult, transport::Transport};
use async_channel::Sender;
use async_trait::async_trait;
use url::Url;

// TODO: Temporary, delete
pub struct TestTransport {
    pub sender: Sender<Vec<u8>>,
}

#[async_trait]
impl Transport for TestTransport {
    async fn send_message(&self, msg: Vec<u8>, _service_endpoint: Url) -> VcxResult<()> {
        self.sender.send(msg).await.unwrap();
        Ok(())
    }
}
