use messages::decorators::please_ack::{AckOn, PleaseAck};
use messages::msg_fields::protocols::revocation::ack::AckRevoke;
use messages::msg_fields::protocols::revocation::revoke::{RevocationFormat, Revoke, RevokeContent, RevokeDecorators};
use shared_vcx::maybe_known::MaybeKnown;
use uuid::Uuid;

use crate::errors::error::prelude::*;
use crate::handlers::util::verify_thread_id;
use crate::protocols::revocation_notification::sender::states::finished::FinishedState;
use crate::protocols::revocation_notification::sender::states::initial::InitialState;
use crate::protocols::revocation_notification::sender::states::sent::NotificationSentState;
use crate::protocols::SendClosure;

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

    pub fn get_notification(&self) -> VcxResult<Revoke> {
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

                let id = Uuid::new_v4().to_string();

                let content = RevokeContent::builder()
                    .credential_id(format!("{rev_reg_id}::{cred_rev_id}"))
                    .revocation_format(MaybeKnown::Known(RevocationFormat::IndyAnoncreds));

                let content = if let Some(comment) = comment {
                    content.comment(comment).build()
                } else {
                    content.build()
                };

                let decorators = RevokeDecorators::builder()
                    .please_ack(PleaseAck::builder().on(ack_on).build())
                    .build();

                let rev_msg: Revoke = Revoke::builder().id(id).content(content).decorators(decorators).build();
                send_message(rev_msg.clone().into()).await?;

                let is_finished = !rev_msg
                    .decorators
                    .please_ack
                    .as_ref()
                    .map(|d| d.on.is_empty())
                    .unwrap_or(false);

                if is_finished {
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

    pub fn handle_ack(self, ack: AckRevoke) -> VcxResult<Self> {
        let state = match self.state {
            SenderFullState::NotificationSent(state)
                if state
                    .get_notification()
                    .decorators
                    .please_ack
                    .as_ref()
                    .map(|d| d.on.is_empty())
                    .unwrap_or(false) =>
            {
                verify_thread_id(&state.get_thread_id(), &ack.clone().into())?;
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

#[allow(clippy::unwrap_used)]
pub mod test_utils {
    use crate::protocols::revocation_notification::test_utils::{_comment, _cred_rev_id, _rev_reg_id};

    use super::*;

    pub fn _sender_config(ack_on: Vec<AckOn>) -> SenderConfig {
        SenderConfigBuilder::default()
            .rev_reg_id(_rev_reg_id())
            .cred_rev_id(_cred_rev_id())
            .comment(Some(_comment()))
            .ack_on(ack_on)
            .build()
            .unwrap()
    }

    pub fn _sender() -> RevocationNotificationSenderSM {
        RevocationNotificationSenderSM::create()
    }
}

// #[cfg(test)]
// pub mod unit_tests {
//     use messages::concepts::ack::test_utils::{_ack, _ack_1};

//     use crate::protocols::revocation_notification::{
//         sender::state_machine::test_utils::*,
//         test_utils::{_revocation_notification, _send_message},
//     };

//     use super::*;

//     async fn _to_revocation_notification_sent_state() -> RevocationNotificationSenderSM {
//         let sm = _sender()
//             .send(_sender_config(vec![AckOn::Receipt]), _send_message())
//             .await
//             .unwrap();
//         assert_match!(SenderFullState::NotificationSent(_), sm.state);
//         sm
//     }

//     async fn _to_finished_state_via_no_ack() -> RevocationNotificationSenderSM {
//         let sm = _sender().send(_sender_config(vec![]), _send_message()).await.unwrap();
//         assert_match!(SenderFullState::Finished(_), sm.state);
//         sm
//     }

//     async fn _to_finished_state_via_ack() -> RevocationNotificationSenderSM {
//         let sm = _to_revocation_notification_sent_state().await;
//         let sm = sm.handle_ack(_ack()).unwrap();
//         assert_match!(SenderFullState::Finished(_), sm.state);
//         sm
//     }

//     #[tokio::test]
//     async fn test_get_notification_from_notification_sent_state() {
//         let sm = _to_revocation_notification_sent_state().await;
//         assert_eq!(
//             sm.get_notification().unwrap(),
//             _revocation_notification(vec![AckOn::Receipt])
//         );
//     }

//     #[tokio::test]
//     async fn test_get_notification_from_finished_state() {
//         let sm = _to_finished_state_via_no_ack().await;
//         assert_eq!(sm.get_notification().unwrap(), _revocation_notification(vec![]));
//     }

//     #[tokio::test]
//     async fn test_get_thread_id_from_notification_sent_state() {
//         let sm = _to_revocation_notification_sent_state().await;
//         assert!(sm.get_thread_id().is_ok());
//     }

//     #[tokio::test]
//     async fn test_get_thread_id_from_finished_state() {
//         let sm = _to_finished_state_via_no_ack().await;
//         assert!(sm.get_thread_id().is_ok());
//     }

//     #[tokio::test]
//     async fn test_handle_ack_correct_thread_id() {
//         _to_finished_state_via_ack().await;
//     }

//     #[tokio::test]
//     async fn test_handle_ack_fails_incorrect_thread_id() {
//         let sm = _to_revocation_notification_sent_state().await;
//         assert!(sm.handle_ack(_ack_1()).is_err());
//     }

//     #[tokio::test]
//     async fn test_handle_ack_cant_handle_ack_twice() {
//         let sm = _to_revocation_notification_sent_state().await;
//         assert!(sm.handle_ack(_ack()).unwrap().handle_ack(_ack()).is_err());
//     }
// }
