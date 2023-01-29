#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvitedState {
    pub(crate) thread_id: Option<String>,
}

impl InvitedState {
    pub fn new(thread_id: Option<String>) -> Self {
        Self { thread_id }
    }

    pub fn thread_id(&self) -> Option<&str> {
        self.thread_id.as_deref()
    }
}