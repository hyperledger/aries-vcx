use serde::{Deserialize, Serialize};
use url::Url;

use super::InvitationImpl;
use crate::{composite_message::Message, decorators::Timing};

pub type PairwiseInvitation = Message<PairwiseInvitationContent<Url>, PwInvitationDecorators>;
pub type PairwiseDidInvitation = Message<PairwiseInvitationContent<String>, PwInvitationDecorators>;

/// Wrapper that represents a pairwise invitation.
// The wrapping is used so that we expose certain types as an abstraction
// over our internal types.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct PairwiseInvitationContent<T>(pub InvitationImpl<T>);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PwInvitationDecorators {
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
