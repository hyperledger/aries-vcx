use crate::protocols::typestate_con::traits::HandleProblem;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvitedState {
    pub(crate) thread_id: Option<String>,
}

impl InvitedState {
    pub fn new(thread_id: Option<String>) -> Self {
        Self { thread_id }
    }

    /// Unlike other states, the ones implementing the [`crate::protocols::typestate_con::traits::ThreadId`],
    /// this state may or may not have a thread ID, so we're returning an option.
    pub fn opt_thread_id(&self) -> Option<&str> {
        self.thread_id.as_deref()
    }
}

impl HandleProblem for InvitedState {}
