use messages::msg_fields::protocols::connection::invitation::Invitation;

use crate::{
    handlers::util::AnyInvitation,
    protocols::connection::trait_bounds::{HandleProblem, ThreadId},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Invited {
    pub(crate) invitation: AnyInvitation,
}

impl Invited {
    pub fn new(invitation: AnyInvitation) -> Self {
        Self { invitation }
    }
}

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

impl HandleProblem for Invited {}
