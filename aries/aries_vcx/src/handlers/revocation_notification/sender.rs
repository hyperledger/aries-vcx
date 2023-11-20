use messages::msg_fields::protocols::revocation::ack::AckRevoke;

use crate::{
    errors::error::prelude::*,
    protocols::{
        revocation_notification::sender::state_machine::{
            RevocationNotificationSenderSM, SenderConfig,
        },
        SendClosure,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevocationNotificationSender {
    sender_sm: RevocationNotificationSenderSM,
}

impl RevocationNotificationSender {
    pub fn build() -> Self {
        Self {
            sender_sm: RevocationNotificationSenderSM::create(),
        }
    }

    pub async fn send_revocation_notification(
        self,
        config: SenderConfig,
        send_message: SendClosure<'_>,
    ) -> VcxResult<Self> {
        let sender_sm = self.sender_sm.send(config, send_message).await?;
        Ok(Self { sender_sm })
    }

    pub async fn handle_revocation_notification_ack(self, ack: AckRevoke) -> VcxResult<Self> {
        let sender_sm = self.sender_sm.handle_ack(ack)?;
        Ok(Self { sender_sm })
    }
}
