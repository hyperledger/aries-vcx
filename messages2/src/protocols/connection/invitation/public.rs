use serde::{Deserialize, Serialize};
use transitive::TransitiveFrom;

use crate::{aries_message::AriesMessage, protocols::connection::Connection};

use super::Invitation;

/// Represents a public invitation.
#[derive(Clone, Debug, Deserialize, Serialize, TransitiveFrom)]
#[transitive(into(Invitation, Connection, AriesMessage))]
pub struct PublicInvitation {
    #[serde(rename = "@id")]
    pub id: String,
    pub label: String,
    pub did: String,
}
