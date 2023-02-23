use serde::{Deserialize, Serialize};

/// Represents a public invitation.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublicInvitation {
    #[serde(rename = "@id")]
    pub id: String,
    pub label: String,
    pub did: String,
}
