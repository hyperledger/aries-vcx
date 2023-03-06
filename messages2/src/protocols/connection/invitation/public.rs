use serde::{Deserialize, Serialize};

use crate::composite_message::Message;

pub type PublicInvitation = Message<PublicInvitationContent>;

/// Represents a public invitation.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicInvitationContent {
    pub label: String,
    pub did: String,
}
