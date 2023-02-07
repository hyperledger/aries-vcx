use messages::{diddoc::aries::diddoc::AriesDidDoc, protocols::connection::invite::Invitation};

use crate::protocols::connection::trait_bounds::{TheirDidDoc, ThreadId, BootstrapDidDoc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Invited {
    pub(crate) did_doc: AriesDidDoc,
    pub(crate) invitation: Invitation,
}

impl Invited {
    pub fn new(did_doc: AriesDidDoc, invitation: Invitation) -> Self {
        Self { did_doc, invitation }
    }
}

impl TheirDidDoc for Invited {
    fn their_did_doc(&self) -> &AriesDidDoc {
        &self.did_doc
    }
}

impl BootstrapDidDoc for Invited {}

impl ThreadId for Invited {
    fn thread_id(&self) -> &str {
        self.invitation.get_id()
    }
}
