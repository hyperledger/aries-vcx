use messages::protocols::connection::invite::Invitation;

use crate::protocols::connection::trait_bounds::ThreadId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Initial {
    pub(crate) invitation: Invitation,
}

impl Initial {
    pub fn new(invitation: Invitation) -> Self {
        Self { invitation }
    }
}

impl ThreadId for Initial {
    fn thread_id(&self) -> &str {
        self.invitation.get_id()
    }
}
