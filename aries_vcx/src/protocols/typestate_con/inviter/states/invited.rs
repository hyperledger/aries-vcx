#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvitedState {
    pub thread_id: Option<String>,
}

impl InvitedState {
    pub fn new(thread_id: Option<String>) -> Self {
        Self { thread_id }
    }
}
