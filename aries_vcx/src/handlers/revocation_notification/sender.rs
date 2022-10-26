use messages::ack::please_ack::AckOn;

use crate::error::prelude::*;
use crate::protocols::SendClosure;
use crate::protocols::revocation_notification::sender::state_machine::{RevocationNotificationSenderSM, SenderConfigBuilder};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevocationNotificationSender {
    sender_sm: RevocationNotificationSenderSM,
}

impl RevocationNotificationSender {
    pub fn build(rev_reg_id: String, cred_rev_id: String, ack_on: Vec<AckOn>, comment: Option<String>) -> VcxResult<Self> {
        let config = SenderConfigBuilder::default()
            .rev_reg_id(rev_reg_id)
            .cred_rev_id(cred_rev_id)
            .ack_on(ack_on)
            .build()?;
        Ok(Self { sender_sm: RevocationNotificationSenderSM::create(config) })
    }

    pub async fn send_revocation_notification(self, send_message: SendClosure) -> VcxResult<Self> {
        let sender_sm = self.sender_sm.send(send_message).await?;
        Ok(Self { sender_sm })
    }
}
