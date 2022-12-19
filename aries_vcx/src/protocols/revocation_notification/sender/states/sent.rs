use messages::protocols::revocation_notification::revocation_notification::RevocationNotification;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NotificationSentState {
    rev_msg: RevocationNotification,
}

impl NotificationSentState {
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
