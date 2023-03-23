pub mod pairwise;
pub mod public;

use derive_more::From;
use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};
use url::Url;

pub use self::{
    pairwise::{PairwiseDidInvitation, PairwiseInvitation, PairwiseInvitationContent, PwInvitationDecorators},
    public::{PublicInvitation, PublicInvitationContent},
};

use super::Connection;
use crate::{
    misc::utils::transit_to_aries_msg,
    msg_types::types::connection::ConnectionV1_0,
    protocols::traits::{MessageContent, MessageWithKind},
};

#[derive(Debug, Clone, From, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "ConnectionV1_0::Invitation")]
#[serde(untagged)]
pub enum Invitation {
    Public(PublicInvitation),
    Pairwise(PairwiseInvitation),
    PairwiseDID(PairwiseDidInvitation),
}

// We implement the message kind on this type as we have to rely on
// untagged deserialization, since we cannot know the invitation format
// ahead of time.
//
// However, to have the capability of setting different decorators
// based on the invitation format, we don't wrap the [`Invitation`]
// in a [`Message`], but rather its variants.
//
// This means that we cannot resolve the message kind through the
// generic `Message<C: MessageContent, D>` because, in this case,
// the variants don't implement `MessageContent`.
//
// Hence, the manual impl below.
impl MessageWithKind for Invitation {
    type MsgKind = <Self as MessageContent>::Kind;

    fn msg_kind() -> Self::MsgKind {
        Self::kind()
    }
}

transit_to_aries_msg!(PublicInvitationContent, Invitation, Connection);
transit_to_aries_msg!(PairwiseInvitationContent<Url>:PwInvitationDecorators, Invitation, Connection);
transit_to_aries_msg!(PairwiseInvitationContent<String>:PwInvitationDecorators, Invitation, Connection);
