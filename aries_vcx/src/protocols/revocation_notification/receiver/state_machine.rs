use messages::ack::please_ack::AckOn;
use messages::issuance::revocation_ack::RevocationAck;
use messages::issuance::revocation_notification::RevocationNotification;

use crate::error::prelude::*;
use crate::protocols::SendClosure;
use crate::protocols::revocation_notification::receiver::states::initial::InitialState;
use crate::protocols::revocation_notification::receiver::states::received::NotificationReceivedState;
use crate::protocols::revocation_notification::receiver::states::finished::FinishedState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RevocationNotificationReceiverSM {
    state: ReceiverFullState
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReceiverFullState {
    Initial(InitialState),
    NotificationReceived(NotificationReceivedState),
    Finished(FinishedState),
}

impl RevocationNotificationReceiverSM {
    pub fn create() -> Self {
        Self {
            state: ReceiverFullState::Initial(InitialState::new())
        }
    }

    pub fn get_notification(&self) -> VcxResult<RevocationNotification> {
        match &self.state {
            ReceiverFullState::NotificationReceived(state) => Ok(state.get_notification()),
            ReceiverFullState::Finished(state) => Ok(state.get_notification()),
            _ => { return Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Revocation notification not yet known in this state")); }
        }
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        match &self.state {
            ReceiverFullState::NotificationReceived(state) => Ok(state.get_thread_id()),
            ReceiverFullState::Finished(state) => Ok(state.get_thread_id()),
            _ => { return Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Thread ID not yet known in this state")); }
        }
    }

    pub async fn handle_revocation_notification(self, notification: RevocationNotification, send_message: SendClosure) -> VcxResult<Self> {
        let state = match self.state {
            ReceiverFullState::NotificationReceived(_) |
                ReceiverFullState::Finished(_) => {
                if notification.ack_on(AckOn::Receipt) {
                    let ack = RevocationAck::create().set_thread_id(&self.get_thread_id()?).set_out_time();
                    send_message(ack.to_a2a_message()).await?;
                    ReceiverFullState::Finished(FinishedState::new(notification))
                } else {
                    ReceiverFullState::NotificationReceived(NotificationReceivedState::new(notification))
                }
            }
            _ => { return Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Ack already received")); }
        };
        Ok(Self { state, ..self })
    }

    pub async fn send_ack(self, send_message: SendClosure) -> VcxResult<Self> {
        let state = match self.state {
            ReceiverFullState::NotificationReceived(_) |
                ReceiverFullState::Finished(_) => {
                if !self.get_notification()?.ack_on(AckOn::Outcome) {
                    warn!("Revocation notification should have already been sent or on sent at all");
                }
                let ack = RevocationAck::create().set_thread_id(&self.get_thread_id()?).set_out_time();
                send_message(ack.to_a2a_message()).await?;
                ReceiverFullState::Finished(FinishedState::new(self.get_notification()?))
            }
            _ => { return Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Ack already received")); }
        };
        Ok(Self { state, ..self })
    }
}
