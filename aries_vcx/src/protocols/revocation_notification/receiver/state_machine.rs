use messages::ack::please_ack::AckOn;
use messages::issuance::revocation_ack::RevocationAck;
use messages::issuance::revocation_notification::{RevocationNotification, RevocationFormat};

use crate::error::prelude::*;
use crate::protocols::SendClosure;
use crate::protocols::revocation_notification::receiver::states::initial::InitialState;
use crate::protocols::revocation_notification::receiver::states::received::NotificationReceivedState;
use crate::protocols::revocation_notification::receiver::states::finished::FinishedState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RevocationNotificationReceiverSM {
    state: ReceiverFullState,
    rev_reg_id: String,
    cred_rev_id: String
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
            cred_rev_id
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
                self.validate_revocation_notification(&notification)?;
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

    fn validate_revocation_notification(&self, notification: &RevocationNotification) -> VcxResult<()> {
        let check_rev_format = || -> VcxResult<()> {
            if notification.get_revocation_format() != RevocationFormat::IndyAnoncreds {
                Err(VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails, "Received revocation notification with unsupported revocation format, only IndyAnoncreds supported"))
            } else {
                Ok(())
            }
        };

        let cred_id = notification.get_credential_id();
        let parts = cred_id.split("::").collect::<Vec<&str>>();
        let check_rev_reg_id = |()| -> VcxResult<()> {
            if let Some(rev_reg_id) = parts.get(0) {
                if rev_reg_id.to_string() != self.rev_reg_id {
                    Err(VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails, "Revocation registry ID in received notification does not match revocation registry ID of this credential"))
                } else {
                    Ok(())
                }
            } else {
                Err(VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails, "Invalid credential ID, missing revocation registry ID"))
            }
        };
        let check_cred_rev_id = |()| -> VcxResult<()> {
            if let Some(cred_rev_id) = parts.get(1) {
                if cred_rev_id.to_string() != self.cred_rev_id {
                    Err(VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails, "Credential revocation ID in received notification does not match revocation ID of this credential"))
                } else {
                    Ok(())
                }
            } else {
                Err(VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails, "Invalid credential ID, missing revocation registry ID"))
            }
        };

        check_rev_format()
            .and_then(check_rev_reg_id)
            .and_then(check_cred_rev_id)
    }
}
