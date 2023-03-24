pub mod pairwise;
pub mod public;

use derive_more::From;
use serde::{Deserialize, Serialize};
use url::Url;

pub use self::{
    pairwise::{PairwiseDidInvitation, PairwiseInvitation, PairwiseInvitationContent, PwInvitationDecorators},
    public::{PublicInvitation, PublicInvitationContent},
};

use super::Connection;
use crate::{
    misc::utils::{transit_to_aries_msg},
};

#[derive(Debug, Clone, From, Deserialize, Serialize,  PartialEq)]
#[serde(untagged)]
pub enum Invitation {
    Public(PublicInvitation),
    Pairwise(PairwiseInvitation),
    PairwiseDID(PairwiseDidInvitation),
}

transit_to_aries_msg!(PublicInvitationContent, Invitation, Connection);
transit_to_aries_msg!(PairwiseInvitationContent<Url>:PwInvitationDecorators, Invitation, Connection);
transit_to_aries_msg!(PairwiseInvitationContent<String>:PwInvitationDecorators, Invitation, Connection);