use diddoc::aries::diddoc::AriesDidDoc;
use messages2::msg_fields::protocols::connection::invitation::Invitation;

use crate::{protocols::connection::trait_bounds::{BootstrapDidDoc, TheirDidDoc, ThreadId}, handlers::util::AnyInvitation};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Invited {
    pub(crate) did_doc: AriesDidDoc,
    pub(crate) invitation: AnyInvitation,
}

impl Invited {
    pub fn new(did_doc: AriesDidDoc, invitation: AnyInvitation) -> Self {
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
        match self.invitation {
            AnyInvitation::Con(Invitation::Public(i)) => i.id.as_str(),
            AnyInvitation::Con(Invitation::Pairwise(i)) => i.id.as_str(),
            AnyInvitation::Con(Invitation::PairwiseDID(i)) => i.id.as_str(),
            AnyInvitation::Oob(i) => i.id.as_str(),
        }
    }
}
