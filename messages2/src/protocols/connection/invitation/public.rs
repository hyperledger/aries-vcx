use serde::{Deserialize, Serialize};

/// Represents a public invitation.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicInvitation {
    pub label: String,
    pub did: String,
}
