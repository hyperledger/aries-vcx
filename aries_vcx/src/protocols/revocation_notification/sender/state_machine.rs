use messages::ack::please_ack::AckOn;
use messages::issuance::revocation_ack::RevocationAck;
use messages::issuance::revocation_notification::{RevocationNotification, RevocationFormat};

use crate::error::prelude::*;
use crate::protocols::SendClosure;
use crate::protocols::revocation_notification::sender::states::initial::InitialState;
use crate::protocols::revocation_notification::sender::states::sent::NotificationSentState;
use crate::protocols::revocation_notification::sender::states::finished::FinishedState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RevocationNotificationSenderSM {
    state: SenderFullState
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
    ack_on: Vec<AckOn>
}

impl RevocationNotificationSenderSM {
    pub fn create() -> Self {
        Self {
            state: SenderFullState::Initial(InitialState::new())
        }
    }

    pub fn get_notification(&self) -> VcxResult<RevocationNotification> {
        match &self.state {
            SenderFullState::NotificationSent(state) => Ok(state.get_notification()),
            SenderFullState::Finished(state) => Ok(state.get_notification()),
            _ => { return Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Revocation notification not yet known in this state")); }
        }
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        match &self.state {
            SenderFullState::NotificationSent(state) => Ok(state.get_thread_id()),
            SenderFullState::Finished(state) => Ok(state.get_thread_id()),
            _ => { return Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Thread ID not yet known in this state")); }
        }
    }

    pub async fn send(self, config: SenderConfig, send_message: SendClosure) -> VcxResult<Self> {
        let state = match self.state {
            SenderFullState::Initial(_) |
                SenderFullState::NotificationSent(_) => {
                let SenderConfig { rev_reg_id, cred_rev_id, comment, ack_on } = config;
                let rev_msg = RevocationNotification::create()
                    .set_credential_id(rev_reg_id, cred_rev_id)
                    .set_ack_on(ack_on)
                    .set_comment(comment)
                    .set_revocation_format(RevocationFormat::IndyAnoncreds);
                send_message(self.get_notification()?.to_a2a_message()).await?;
                if rev_msg.ack_on_any() {
                    SenderFullState::Finished(FinishedState::new(rev_msg, None))
                } else {
                    SenderFullState::NotificationSent(NotificationSentState::new(rev_msg))
                }
            }
            _ => { return Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Ack already received")); }
        };
        Ok(Self { state, ..self })
    }

    pub fn handle_ack(self, ack: RevocationAck) -> VcxResult<Self> {
        let state = match self.state {
            SenderFullState::NotificationSent(state) if state.get_notification().ack_on_any() => {
                SenderFullState::Finished(FinishedState::new(state.get_notification(), Some(ack)))
            }
            _ => { return Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Ack not expected in this state")); }
        };
        Ok(Self { state, ..self })
    }
}
