use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use super::InvitationImpl;
use crate::message_type::message_family::connection::ConnectionV1_0;
use crate::protocols::traits::MessageKind;

/// Wrapper that represents a pairwise invitation.
// The wrapping is used so that we expose certain types as an abstraction
// over our internal types.
#[derive(Debug, Clone, Deserialize, Serialize, MessageContent)]
#[message(kind = "ConnectionV1_0::Invitation")]
#[serde(transparent)]
pub struct PairwiseInvitation<T>(pub InvitationImpl<T>);
