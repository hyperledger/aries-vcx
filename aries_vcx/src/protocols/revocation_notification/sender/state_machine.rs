use messages::ack::please_ack::AckOn;
use messages::issuance::revocation_ack::RevocationAck;
use messages::issuance::revocation_notification::RevocationNotification;

use crate::error::prelude::*;
use crate::protocols::SendClosure;
use crate::protocols::revocation_notification::sender::states::initial::InitialState;
use crate::protocols::revocation_notification::sender::states::sent::NotificationSentState;
use crate::protocols::revocation_notification::sender::states::finished::FinishedState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RevocationNotificationSenderSM {
    state: SenderFullState,
    thread_id: String,
    rev_msg: RevocationNotification,
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
    pub fn create(config: SenderConfig) -> Self {
        // TODO: Move to the point of sending
        let SenderConfig { rev_reg_id, cred_rev_id, comment, .. } = config;
        let rev_msg = RevocationNotification::create()
            .set_credential_id(rev_reg_id, cred_rev_id)
            .set_comment(comment);
        Self {
            state: SenderFullState::Initial(InitialState::new()),
            thread_id: rev_msg.get_thread_id(),
            rev_msg,
        }
    }

    pub async fn send(self, send_message: SendClosure) -> VcxResult<Self> {
        let state = match self.state {
            SenderFullState::Initial(_) |
                SenderFullState::NotificationSent(_) => {
                send_message(self.rev_msg.to_a2a_message()).await?;
                SenderFullState::NotificationSent(NotificationSentState::new())
            }
            _ => { return Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Ack already received")); }
        };
        Ok(Self { state, ..self })
    }

    pub fn handle_ack(self, ack: RevocationAck) -> VcxResult<Self> {
        let state = match self.state {
            SenderFullState::NotificationSent(state) => {
                // TODO: Check if ack was required
                SenderFullState::Finished(FinishedState::new())
            }
            _ => { return Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Ack not expected in this state")); }
        };
        Ok(Self { state, ..self })
    }
}
