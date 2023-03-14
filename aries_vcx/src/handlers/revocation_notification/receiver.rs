use messages::protocols::revocation_notification::revocation_notification::RevocationNotification;

use crate::{
    errors::error::prelude::*,
    protocols::{revocation_notification::receiver::state_machine::RevocationNotificationReceiverSM, SendClosure},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevocationNotificationReceiver {
    receiver_sm: RevocationNotificationReceiverSM,
}

impl RevocationNotificationReceiver {
    pub fn build(rev_reg_id: String, cred_rev_id: String) -> Self {
        Self {
            receiver_sm: RevocationNotificationReceiverSM::create(rev_reg_id, cred_rev_id),
        }
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        self.receiver_sm.get_thread_id()
    }

    pub async fn handle_revocation_notification(
        self,
        notification: RevocationNotification,
        send_message: SendClosure,
    ) -> VcxResult<Self> {
        let receiver_sm = self
            .receiver_sm
            .handle_revocation_notification(notification, send_message)
            .await?;
        Ok(Self { receiver_sm })
    }

    pub async fn send_ack(self, send_message: SendClosure) -> VcxResult<Self> {
        let receiver_sm = self.receiver_sm.send_ack(send_message).await?;
        Ok(Self { receiver_sm })
    }
}
