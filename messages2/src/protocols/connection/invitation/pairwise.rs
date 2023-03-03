use serde::{Deserialize, Serialize};

use super::InvitationImpl;
use crate::decorators::Timing;

/// Wrapper that represents a pairwise invitation.
// The wrapping is used so that we expose certain types as an abstraction
// over our internal types.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct PairwiseInvitation<T>(pub InvitationImpl<T>);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PwInvitationDecorators {
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
