pub mod pairwise;
pub mod public;

use derive_more::From;
use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};
use url::Url;

pub use self::{
    pairwise::{PairwiseDidInvitation, PairwiseInvitation},
    public::PublicInvitation,
};
use self::{
    pairwise::{PairwiseInvitationContent, PwInvitationDecorators},
    public::PublicInvitationContent,
};
use super::Connection;
use crate::{
    misc::utils::transit_to_aries_msg,
    msg_types::types::connection::ConnectionV1_0Kind,
    protocols::traits::{ConcreteMessage, HasKind},
};

/// Type used to encapsulate a fully resolved invitation, which
/// contains all the information necessary for generating a
/// [`crate::protocols::connection::request::Request`].
///
/// Other invitation types would get resolved to this.
// We rely on the URL version of the pairwise invitation because, coincidentally,
// that's what a fully resolved invitation looks like.
// If other fields are needed in the future, this type could be adapted.
pub struct CompleteInvitationContent(PairwiseInvitationContent<Url>);

// We implement the message kind on this type as we have to rely on
// untagged deserialization, since we cannot know the invitation format
// ahead of time.
//
// However, to have the capability of setting different decorators
// based on the invitation format, we don't wrap the [`Invitation`]
// in a [`Message`], but rather its variants.
#[derive(Debug, Clone, From, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "ConnectionV1_0Kind::Invitation")]
#[serde(untagged)]
pub enum Invitation {
    Public(PublicInvitation),
    Pairwise(PairwiseInvitation),
    PairwiseDID(PairwiseDidInvitation),
}

impl HasKind for Invitation {
    type KindType = <Self as ConcreteMessage>::Kind;

    fn kind_type() -> Self::KindType {
        Self::kind()
    }
}

transit_to_aries_msg!(PublicInvitationContent, Invitation, Connection);
transit_to_aries_msg!(PairwiseInvitationContent<Url>:PwInvitationDecorators, Invitation, Connection);
transit_to_aries_msg!(PairwiseInvitationContent<String>:PwInvitationDecorators, Invitation, Connection);
