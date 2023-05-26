use did_doc::schema::did_doc::DidDocument;
use did_resolver_sov::resolution::ExtraFieldsSov;
use messages::msg_fields::protocols::connection::invitation::Invitation;

use crate::{
    handlers::util::AnyInvitation,
    protocols::connection::trait_bounds::{BootstrapDidDoc, TheirDidDoc, ThreadId},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Invited {
    pub(crate) did_doc: DidDocument<ExtraFieldsSov>,
    pub(crate) invitation: AnyInvitation,
}

impl Invited {
    pub fn new(did_doc: DidDocument<ExtraFieldsSov>, invitation: AnyInvitation) -> Self {
        Self { did_doc, invitation }
    }
}

impl TheirDidDoc for Invited {
    fn their_did_doc(&self) -> &DidDocument<ExtraFieldsSov> {
        &self.did_doc
    }
}

impl BootstrapDidDoc for Invited {}

impl ThreadId for Invited {
    fn thread_id(&self) -> &str {
        match &self.invitation {
            AnyInvitation::Con(Invitation::Public(i)) => i.id.as_str(),
            AnyInvitation::Con(Invitation::Pairwise(i)) => i.id.as_str(),
            AnyInvitation::Con(Invitation::PairwiseDID(i)) => i.id.as_str(),
            AnyInvitation::Oob(i) => i.id.as_str(),
        }
    }
}
