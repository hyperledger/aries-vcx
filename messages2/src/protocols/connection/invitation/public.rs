use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::message_type::message_family::connection::ConnectionV1_0;
use crate::protocols::traits::MessageKind;

/// Represents a public invitation.
#[derive(Debug, Clone, Deserialize, Serialize, MessageContent)]
#[message(kind = "ConnectionV1_0::Invitation")]
pub struct PublicInvitation {
    pub label: String,
    pub did: String,
}
