use messages::{
    concepts::ack::please_ack::AckOn,
    protocols::revocation_notification::{
        revocation_ack::RevocationAck,
        revocation_notification::{RevocationFormat, RevocationNotification},
    },
};

use crate::{
    errors::error::prelude::*,
    handlers::util::verify_thread_id,
    protocols::{
        revocation_notification::sender::states::{
            finished::FinishedState, initial::InitialState, sent::NotificationSentState,
        },
        SendClosure,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RevocationNotificationSenderSM {
    state: SenderFullState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SenderFullState {
    Initial(InitialState),
    NotificationSent(NotificationSentState),
    Finished(FinishedState),
}

#[derive(Default, Builder)]
pub struct SenderConfig {
    rev_reg_id: String,
    cred_rev_id: String,
    comment: Option<String>,
    ack_on: Vec<AckOn>,
}

impl RevocationNotificationSenderSM {
    pub fn create() -> Self {
        Self {
            state: SenderFullState::Initial(InitialState::new()),
        }
    }

    pub fn get_notification(&self) -> VcxResult<RevocationNotification> {
        match &self.state {
            SenderFullState::NotificationSent(state) => Ok(state.get_notification()),
            SenderFullState::Finished(state) => Ok(state.get_notification()),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Revocation notification not yet known in this state",
            )),
        }
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        match &self.state {
            SenderFullState::NotificationSent(state) => Ok(state.get_thread_id()),
            SenderFullState::Finished(state) => Ok(state.get_thread_id()),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Thread ID not yet known in this state",
            )),
        }
    }

    pub async fn send(self, config: SenderConfig, send_message: SendClosure) -> VcxResult<Self> {
        let state = match self.state {
            SenderFullState::Initial(_) | SenderFullState::NotificationSent(_) => {
                let SenderConfig {
                    rev_reg_id,
                    cred_rev_id,
                    comment,
                    ack_on,
                } = config;
                let rev_msg = RevocationNotification::create()
                    .set_credential_id(rev_reg_id, cred_rev_id)
                    .set_ack_on(ack_on)
                    .set_comment(comment)
                    .set_revocation_format(RevocationFormat::IndyAnoncreds);
                send_message(rev_msg.to_a2a_message()).await?;
                if !rev_msg.ack_on_any() {
                    SenderFullState::Finished(FinishedState::new(rev_msg, None))
                } else {
                    SenderFullState::NotificationSent(NotificationSentState::new(rev_msg))
                }
            }
            _ => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Ack already received",
                ));
            }
        };
        Ok(Self { state })
    }

    pub fn handle_ack(self, ack: RevocationAck) -> VcxResult<Self> {
        let state = match self.state {
            SenderFullState::NotificationSent(state) if state.get_notification().ack_on_any() => {
                verify_thread_id(&state.get_thread_id(), &ack.to_a2a_message())?;
                SenderFullState::Finished(FinishedState::new(state.get_notification(), Some(ack)))
            }
            _ => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Ack not expected in this state",
                ));
            }
        };
        Ok(Self { state })
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use super::*;
    use crate::protocols::revocation_notification::test_utils::{_comment, _cred_rev_id, _rev_reg_id};

    pub fn _sender_config(ack_on: Vec<AckOn>) -> SenderConfig {
        SenderConfigBuilder::default()
            .rev_reg_id(_rev_reg_id())
            .cred_rev_id(_cred_rev_id())
            .comment(_comment())
            .ack_on(ack_on)
            .build()
            .unwrap()
    }

    pub fn _sender() -> RevocationNotificationSenderSM {
        RevocationNotificationSenderSM::create()
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use messages::concepts::ack::test_utils::{_ack, _ack_1};

    use super::*;
    use crate::protocols::revocation_notification::{
        sender::state_machine::test_utils::*,
        test_utils::{_revocation_notification, _send_message},
    };

    async fn _to_revocation_notification_sent_state() -> RevocationNotificationSenderSM {
        let sm = _sender()
            .send(_sender_config(vec![AckOn::Receipt]), _send_message())
            .await
            .unwrap();
        assert_match!(SenderFullState::NotificationSent(_), sm.state);
        sm
    }

    async fn _to_finished_state_via_no_ack() -> RevocationNotificationSenderSM {
        let sm = _sender().send(_sender_config(vec![]), _send_message()).await.unwrap();
        assert_match!(SenderFullState::Finished(_), sm.state);
        sm
    }

    async fn _to_finished_state_via_ack() -> RevocationNotificationSenderSM {
        let sm = _to_revocation_notification_sent_state().await;
        let sm = sm.handle_ack(_ack()).unwrap();
        assert_match!(SenderFullState::Finished(_), sm.state);
        sm
    }

    #[tokio::test]
    async fn test_get_notification_from_notification_sent_state() {
        let sm = _to_revocation_notification_sent_state().await;
        assert_eq!(
            sm.get_notification().unwrap(),
            _revocation_notification(vec![AckOn::Receipt])
        );
    }

    #[tokio::test]
    async fn test_get_notification_from_finished_state() {
        let sm = _to_finished_state_via_no_ack().await;
        assert_eq!(sm.get_notification().unwrap(), _revocation_notification(vec![]));
    }

    #[tokio::test]
    async fn test_get_thread_id_from_notification_sent_state() {
        let sm = _to_revocation_notification_sent_state().await;
        assert!(sm.get_thread_id().is_ok());
    }

    #[tokio::test]
    async fn test_get_thread_id_from_finished_state() {
        let sm = _to_finished_state_via_no_ack().await;
        assert!(sm.get_thread_id().is_ok());
    }

    #[tokio::test]
    async fn test_handle_ack_correct_thread_id() {
        _to_finished_state_via_ack().await;
    }

    #[tokio::test]
    async fn test_handle_ack_fails_incorrect_thread_id() {
        let sm = _to_revocation_notification_sent_state().await;
        assert!(sm.handle_ack(_ack_1()).is_err());
    }

    #[tokio::test]
    async fn test_handle_ack_cant_handle_ack_twice() {
        let sm = _to_revocation_notification_sent_state().await;
        assert!(sm.handle_ack(_ack()).unwrap().handle_ack(_ack()).is_err());
    }
}
