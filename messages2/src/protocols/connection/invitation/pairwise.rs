use serde::{Deserialize, Serialize};
use url::Url;

use crate::{composite_message::Message, decorators::Timing};

pub type PairwiseInvitation = Message<PairwiseInvitationContent<Url>, PwInvitationDecorators>;
pub type PairwiseDidInvitation = Message<PairwiseInvitationContent<String>, PwInvitationDecorators>;

/// Wrapper that represents a pairwise invitation.
// The wrapping is used so that we expose certain types as an abstraction
// over our internal types.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairwiseInvitationContent<T> {
    pub label: String,
    pub recipient_keys: Vec<String>,
    #[serde(default)]
    pub routing_keys: Vec<String>,
    pub service_endpoint: T,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PwInvitationDecorators {
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
