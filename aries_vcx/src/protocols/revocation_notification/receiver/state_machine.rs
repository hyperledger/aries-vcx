use chrono::Utc;
use messages::decorators::please_ack::AckOn;
use messages::decorators::thread::Thread;
use messages::decorators::timing::Timing;
use messages::msg_fields::protocols::notification::ack::{AckContent, AckDecorators, AckStatus};
use messages::msg_fields::protocols::revocation::ack::AckRevoke;
use messages::msg_fields::protocols::revocation::revoke::{RevocationFormat, Revoke};
use shared_vcx::maybe_known::MaybeKnown;
use uuid::Uuid;

use crate::errors::error::prelude::*;
use crate::protocols::revocation_notification::receiver::states::finished::FinishedState;
use crate::protocols::revocation_notification::receiver::states::initial::InitialState;
use crate::protocols::revocation_notification::receiver::states::received::NotificationReceivedState;
use crate::protocols::SendClosure;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RevocationNotificationReceiverSM {
    state: ReceiverFullState,
    rev_reg_id: String,
    cred_rev_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReceiverFullState {
    Initial(InitialState),
    NotificationReceived(NotificationReceivedState),
    Finished(FinishedState),
}

impl RevocationNotificationReceiverSM {
    pub fn create(rev_reg_id: String, cred_rev_id: String) -> Self {
        Self {
            state: ReceiverFullState::Initial(InitialState::new()),
            rev_reg_id,
            cred_rev_id,
        }
    }

    pub fn get_notification(&self) -> VcxResult<Revoke> {
        match &self.state {
            ReceiverFullState::NotificationReceived(state) => Ok(state.get_notification()),
            ReceiverFullState::Finished(state) => Ok(state.get_notification()),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Revocation notification not yet known in this state",
            )),
        }
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        match &self.state {
            ReceiverFullState::NotificationReceived(state) => Ok(state.get_thread_id()),
            ReceiverFullState::Finished(state) => Ok(state.get_thread_id()),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Thread ID not yet known in this state",
            )),
        }
    }

    pub async fn handle_revocation_notification(
        self,
        notification: Revoke,
        send_message: SendClosure,
    ) -> VcxResult<Self> {
        let state = match self.state {
            ReceiverFullState::Initial(_) => {
                self.validate_revocation_notification(&notification)?;
                if !notification
                    .decorators
                    .please_ack
                    .as_ref()
                    .map(|d| d.on.is_empty())
                    .unwrap_or(false)
                {
                    ReceiverFullState::Finished(FinishedState::new(notification))
                } else if notification
                    .decorators
                    .please_ack
                    .as_ref()
                    .map(|d| d.on.contains(&AckOn::Receipt))
                    .unwrap_or(false)
                {
                    let id = Uuid::new_v4().to_string();
                    let content = AckContent::builder().status(AckStatus::Ok).build();

                    let thread_id = notification
                        .decorators
                        .thread
                        .as_ref()
                        .map(|t| t.thid.clone())
                        .unwrap_or(notification.id.clone());

                    let decorators = AckDecorators::builder()
                        .thread(Thread::builder().thid(thread_id).build())
                        .timing(Timing::builder().out_time(Utc::now()).build())
                        .build();

                    let ack = AckRevoke::builder()
                        .id(id)
                        .content(content)
                        .decorators(decorators)
                        .build();

                    send_message(ack).await?;
                    ReceiverFullState::Finished(FinishedState::new(notification))
                } else {
                    ReceiverFullState::NotificationReceived(NotificationReceivedState::new(notification))
                }
            }
            _ => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Ack already received",
                ));
            }
        };
        Ok(Self { state, ..self })
    }

    pub async fn send_ack(self, send_message: SendClosure) -> VcxResult<Self> {
        let state = match self.state {
            ReceiverFullState::NotificationReceived(_) | ReceiverFullState::Finished(_) => {
                let notification = self.get_notification()?;

                if !notification
                    .decorators
                    .please_ack
                    .as_ref()
                    .map(|d| d.on.contains(&AckOn::Outcome))
                    .unwrap_or(false)
                {
                    warn!("Revocation notification should have already been sent or not sent at all");
                }

                let id = Uuid::new_v4().to_string();
                let content = AckContent::builder().status(AckStatus::Ok).build();

                let thread_id = notification
                    .decorators
                    .thread
                    .as_ref()
                    .map(|t| t.thid.clone())
                    .unwrap_or(notification.id.clone());

                let decorators = AckDecorators::builder()
                    .thread(Thread::builder().thid(thread_id).build())
                    .timing(Timing::builder().out_time(Utc::now()).build())
                    .build();

                let ack = AckRevoke::builder()
                    .id(id)
                    .content(content)
                    .decorators(decorators)
                    .build();

                send_message(ack).await?;
                ReceiverFullState::Finished(FinishedState::new(notification))
            }
            _ => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Ack already sent",
                ));
            }
        };
        Ok(Self { state, ..self })
    }

    fn validate_revocation_notification(&self, notification: &Revoke) -> VcxResult<()> {
        let check_rev_format = || -> VcxResult<()> {
            if notification.content.revocation_format != MaybeKnown::Known(RevocationFormat::IndyAnoncreds) {
                Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidRevocationDetails,
                    "Received revocation notification with unsupported revocation format, only IndyAnoncreds supported",
                ))
            } else {
                Ok(())
            }
        };

        let cred_id = notification.content.credential_id.clone();
        let parts = cred_id.split("::").collect::<Vec<&str>>();
        let check_rev_reg_id = |()| -> VcxResult<()> {
            if let Some(rev_reg_id) = parts.first() {
                if *rev_reg_id != self.rev_reg_id {
                    Err(AriesVcxError::from_msg(AriesVcxErrorKind::InvalidRevocationDetails, "Revocation registry ID in received notification does not match revocation registry ID of this credential"))
                } else {
                    Ok(())
                }
            } else {
                Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidRevocationDetails,
                    "Invalid credential ID, missing revocation registry ID",
                ))
            }
        };
        let check_cred_rev_id = |()| -> VcxResult<()> {
            if let Some(cred_rev_id) = parts.get(1) {
                if *cred_rev_id != self.cred_rev_id {
                    Err(AriesVcxError::from_msg(AriesVcxErrorKind::InvalidRevocationDetails, "Credential revocation ID in received notification does not match revocation ID of this credential"))
                } else {
                    Ok(())
                }
            } else {
                Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidRevocationDetails,
                    "Invalid credential ID, missing revocation registry ID",
                ))
            }
        };

        check_rev_format()
            .and_then(check_rev_reg_id)
            .and_then(check_cred_rev_id)
    }
}

pub mod test_utils {
    use messages::AriesMessage;

    use crate::protocols::revocation_notification::test_utils::{_cred_rev_id, _rev_reg_id};

    use super::*;

    pub fn _receiver() -> RevocationNotificationReceiverSM {
        RevocationNotificationReceiverSM::create(_rev_reg_id(), _cred_rev_id())
    }

    pub fn _send_message_but_fail() -> SendClosure {
        Box::new(|_: AriesMessage| {
            Box::pin(async { Err(AriesVcxError::from_msg(AriesVcxErrorKind::IOError, "Mocked error")) })
        })
    }
}

// #[cfg(test)]
// #[allow(clippy::unwrap_used)]
// pub mod unit_tests {
//     use std::sync::mpsc::sync_channel;

//     use messages::AriesMessage;

//     use crate::protocols::revocation_notification::{
//         receiver::state_machine::test_utils::_receiver,
//         receiver::state_machine::test_utils::*,
//         test_utils::{_cred_rev_id, _rev_reg_id, _revocation_notification, _send_message},
//     };

//     use super::*;

//     async fn _to_revocation_notification_received_state() -> RevocationNotificationReceiverSM {
//         let sm = _receiver()
//             .handle_revocation_notification(_revocation_notification(vec![AckOn::Outcome]), _send_message())
//             .await
//             .unwrap();
//         assert_match!(ReceiverFullState::NotificationReceived(_), sm.state);
//         sm
//     }

//     async fn _to_finished_state_via_ack() -> RevocationNotificationReceiverSM {
//         let sm = _receiver()
//             .handle_revocation_notification(_revocation_notification(vec![AckOn::Receipt]), _send_message())
//             .await
//             .unwrap();
//         assert_match!(ReceiverFullState::Finished(_), sm.state);
//         sm
//     }

//     async fn _to_finished_state_via_no_ack() -> RevocationNotificationReceiverSM {
//         let sm = _receiver()
//             .handle_revocation_notification(_revocation_notification(vec![]), _send_message())
//             .await
//             .unwrap();
//         assert_match!(ReceiverFullState::Finished(_), sm.state);
//         sm
//     }

//     async fn _send_ack_on(ack_on: Vec<AckOn>) {
//         let sm = _receiver()
//             .handle_revocation_notification(_revocation_notification(ack_on), _send_message())
//             .await
//             .unwrap();
//         assert_match!(ReceiverFullState::Finished(_), sm.state);
//         let sm = sm.send_ack(_send_message()).await.unwrap();
//         assert_match!(ReceiverFullState::Finished(_), sm.state);
//     }

//     #[tokio::test]
//     async fn test_handle_revocation_notification_sends_ack_when_requested() {
//         let (sender, receiver) = sync_channel(1);
//         let send_message: SendClosure = Box::new(move |_: AriesMessage| {
//             Box::pin(async move {
//                 sender.send(true).unwrap();
//                 VcxResult::Ok(())
//             })
//         });
//         let sm = RevocationNotificationReceiverSM::create(_rev_reg_id(), _cred_rev_id())
//             .handle_revocation_notification(_revocation_notification(vec![AckOn::Receipt]), send_message)
//             .await
//             .unwrap();
//         assert_match!(ReceiverFullState::Finished(_), sm.state);
//         assert!(receiver.recv().unwrap());
//     }

//     #[tokio::test]
//     async fn test_handle_revocation_notification_doesnt_send_ack_when_not_requested() {
//         let sm = RevocationNotificationReceiverSM::create(_rev_reg_id(), _cred_rev_id())
//             .handle_revocation_notification(_revocation_notification(vec![]), _send_message_but_fail())
//             .await
//             .unwrap();
//         assert_match!(ReceiverFullState::Finished(_), sm.state);
//     }

//     #[tokio::test]
//     async fn test_handle_revocation_notification_finishes_ack_requested_and_sent() {
//         _to_finished_state_via_ack().await;
//     }

//     #[tokio::test]
//     async fn test_handle_revocation_notification_finishes_when_ack_not_requested() {
//         _to_finished_state_via_no_ack().await;
//     }

//     #[tokio::test]
//     async fn test_handle_revocation_notification_waits_to_send_ack_on_outcome() {
//         _to_revocation_notification_received_state().await;
//     }

//     #[tokio::test]
//     async fn test_handle_revocation_notification_from_finished_state() {
//         assert!(_to_finished_state_via_ack()
//             .await
//             .handle_revocation_notification(_revocation_notification(vec![]), _send_message())
//             .await
//             .is_err());
//         assert!(_to_finished_state_via_no_ack()
//             .await
//             .handle_revocation_notification(_revocation_notification(vec![]), _send_message())
//             .await
//             .is_err());
//     }

//     #[tokio::test]
//     async fn test_handle_revocation_notification_from_notification_received_state() {
//         assert!(_to_revocation_notification_received_state()
//             .await
//             .handle_revocation_notification(_revocation_notification(vec![]), _send_message())
//             .await
//             .is_err());
//         assert!(_to_revocation_notification_received_state()
//             .await
//             .handle_revocation_notification(_revocation_notification(vec![]), _send_message())
//             .await
//             .is_err());
//     }

//     #[tokio::test]
//     async fn test_send_ack_on_outcome() {
//         assert!(_to_revocation_notification_received_state()
//             .await
//             .send_ack(_send_message())
//             .await
//             .is_ok());
//     }

//     #[tokio::test]
//     async fn test_send_ack_multiple_times_requested() {
//         _send_ack_on(vec![AckOn::Receipt, AckOn::Outcome]).await;
//     }

//     #[tokio::test]
//     async fn test_send_ack_multiple_times_not_requested() {
//         _send_ack_on(vec![AckOn::Receipt]).await;
//     }

//     #[tokio::test]
//     async fn test_send_ack_fails_before_notification_received() {
//         assert!(_receiver().send_ack(_send_message()).await.is_err());
//     }
// }
