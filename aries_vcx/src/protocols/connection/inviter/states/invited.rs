use messages::protocols::connection::invite::Invitation;

use crate::protocols::connection::trait_bounds::{HandleProblem, ThreadId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Invited {
    pub(crate) invitation: Invitation,
}

impl Invited {
    pub fn new(invitation: Invitation) -> Self {
        Self { invitation }
    }
}

impl ThreadId for Invited {
    fn thread_id(&self) -> &str {
        self.invitation.get_id()
    }
}

impl HandleProblem for Invited {}
