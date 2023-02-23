use serde::{Deserialize, Serialize};

use super::InvitationImpl;

/// Wrapper that represents a pairwise invitation.
// The wrapping is used so that we expose certain types in a certain way
// and prevent people
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct PairwiseInvitation<T>(pub InvitationImpl<T>);
