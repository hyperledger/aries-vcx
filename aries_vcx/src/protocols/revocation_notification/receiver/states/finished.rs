use messages2::msg_fields::protocols::revocation::revoke::Revoke;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FinishedState {
    rev_msg: Revoke,
}

impl FinishedState {
    pub fn new(rev_msg: Revoke) -> Self {
        Self { rev_msg }
    }

    pub fn get_notification(&self) -> Revoke {
        self.rev_msg.clone()
    }

    pub fn get_thread_id(&self) -> String {
        self.rev_msg
            .decorators
            .thread
            .map(|t| t.thid.clone())
            .unwrap_or(self.rev_msg.id.clone())
    }
}
