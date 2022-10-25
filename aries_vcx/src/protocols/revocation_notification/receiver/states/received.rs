use messages::issuance::revocation_notification::RevocationNotification;

use crate::protocols::revocation_notification::receiver::state_machine::RevocationNotificationReceiverSM;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NotificationReceivedState {
    rev_msg: RevocationNotification
}

impl NotificationReceivedState {
    pub fn new(rev_msg: RevocationNotification) -> Self {
        Self { rev_msg }
    }

    pub fn get_notification(&self) -> RevocationNotification {
        self.rev_msg.clone()
    }

    pub fn get_thread_id(&self) -> String {
        self.rev_msg.get_thread_id()
    }
}
