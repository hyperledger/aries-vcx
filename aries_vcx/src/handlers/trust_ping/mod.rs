use messages::msg_fields::protocols::trust_ping::{ping::Ping, ping_response::PingResponse};

use super::util::matches_thread_id;
use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    protocols::{trustping::build_ping, SendClosure},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrustPingSender {
    ping: Ping,
    ping_sent: bool,
    response_received: bool,
}

impl TrustPingSender {
    pub fn build(request_response: bool, comment: Option<String>) -> TrustPingSender {
        let ping = build_ping(request_response, comment);
        Self {
            ping,
            ping_sent: false,
            response_received: false,
        }
    }

    pub fn get_ping(&self) -> &Ping {
        &self.ping
    }

    pub fn get_thread_id(&self) -> &str {
        self.ping
            .decorators
            .thread
            .as_ref()
            .map(|t| t.thid.as_str())
            .unwrap_or(self.ping.id.as_str())
    }

    pub async fn send_ping(&mut self, send_message: SendClosure) -> VcxResult<()> {
        if self.ping_sent {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Ping message has already been sent",
            ));
        }
        send_message(self.ping.clone().into()).await?;
        self.ping_sent = true;
        Ok(())
    }

    pub fn handle_ping_response(&mut self, ping: &PingResponse) -> VcxResult<()> {
        if !matches_thread_id!(ping, self.get_thread_id()) {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Thread ID mismatch",
            ));
        }
        if !self.ping.content.response_requested {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Message was not expected",
            ));
        } else {
            self.response_received = true
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod unit_tests {
    use messages::AriesMessage;

    use crate::{
        errors::error::VcxResult,
        handlers::trust_ping::TrustPingSender,
        protocols::{trustping::build_ping_response, SendClosure},
        utils::devsetup::SetupMocks,
    };

    pub fn _send_message() -> SendClosure {
        Box::new(|_: AriesMessage| Box::pin(async { VcxResult::Ok(()) }))
    }

    #[tokio::test]
    async fn test_build_send_ping_process_response() {
        let _setup = SetupMocks::init();
        let mut sender = TrustPingSender::build(true, None);
        sender.send_ping(_send_message()).await.unwrap();
        let ping_response = build_ping_response(&sender.ping);
        sender.handle_ping_response(&ping_response).unwrap();
    }

    #[tokio::test]
    async fn test_should_fail_on_thread_id_mismatch() {
        let _setup = SetupMocks::init();
        let mut sender1 = TrustPingSender::build(true, None);
        let sender2 = TrustPingSender::build(true, None);
        sender1.send_ping(_send_message()).await.unwrap();
        let ping_response = build_ping_response(&sender2.ping);
        sender1.handle_ping_response(&ping_response).unwrap_err();
    }

    #[tokio::test]
    async fn test_should_fail_if_response_was_not_expected() {
        let _setup = SetupMocks::init();
        let mut sender1 = TrustPingSender::build(false, None);
        sender1.send_ping(_send_message()).await.unwrap();
        let ping_response = build_ping_response(&sender1.ping);
        sender1.handle_ping_response(&ping_response).unwrap_err();
    }

    #[test]
    fn test_should_build_ping_with_comment() {
        let _setup = SetupMocks::init();
        let sender1 = TrustPingSender::build(false, Some("hello".to_string()));
        assert_eq!(
            sender1.get_ping().content.comment,
            Some("hello".to_string())
        )
    }
}
