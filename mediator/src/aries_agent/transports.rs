use std::collections::VecDeque;

use async_trait::async_trait;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use log::info;
use serde_json::Value;

#[derive(thiserror::Error, Debug)]
#[error("{msg}")]
pub struct AriesTransportError {
    pub msg: String,
}

impl AriesTransportError {
    fn from_std_error(err: impl std::error::Error) -> Self {
        AriesTransportError {
            msg: err.to_string(),
        }
    }
}

#[async_trait]
pub trait AriesTransport {
    fn pop_aries_envelope(&mut self) -> Result<Value, AriesTransportError>;
    async fn push_aries_envelope(
        &mut self,
        envelope_json: Value,
        destination: &AriesDidDoc,
    ) -> Result<(), AriesTransportError>;
}

pub struct AriesReqwest {
    pub response_queue: VecDeque<Value>,
    pub client: reqwest::Client,
}

#[async_trait]
impl AriesTransport for AriesReqwest {
    async fn push_aries_envelope(
        &mut self,
        envelope_json: Value,
        destination: &AriesDidDoc,
    ) -> Result<(), AriesTransportError> {
        let oob_invited_endpoint = destination
            .get_endpoint()
            .expect("Service needs an endpoint");
        let res = self
            .client
            .post(oob_invited_endpoint)
            .json(&envelope_json)
            .send()
            .await
            .map_err(AriesTransportError::from_std_error)?
            .error_for_status()
            .map_err(AriesTransportError::from_std_error)?;
        let res_json: Value = res
            .json()
            .await
            .map_err(AriesTransportError::from_std_error)?;
        info!("Received aries response{:?}", res_json);
        self.response_queue.push_back(res_json);
        Ok(())
    }
    fn pop_aries_envelope(&mut self) -> Result<Value, AriesTransportError> {
        self.response_queue.pop_front().ok_or(AriesTransportError {
            msg: "No messages in queue".to_owned(),
        })
    }
}
