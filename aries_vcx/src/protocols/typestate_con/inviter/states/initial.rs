use messages::protocols::connection::invite::Invitation;

use crate::protocols::typestate_con::traits::ThreadId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitialState {
    pub(crate) invitation: Invitation,
}

impl InitialState {
    pub fn new(invitation: Invitation) -> Self {
        Self { invitation }
    }
}

impl ThreadId for InitialState {
    fn thread_id(&self) -> &str {
        self.invitation.get_id()
    }
}
