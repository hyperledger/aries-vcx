use serde::{Deserialize, Serialize};
use url::Url;

use crate::{aries_message::AriesMessage, protocols::connection::Connection};

use super::{Invitation, InvitationImpl};

/// Wrapper that represents a pairwise invitation.
// The wrapping is used so that we expose certain types in a certain way
// and prevent people
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct PairwiseInvitation<T>(pub InvitationImpl<T>);

impl From<PairwiseInvitation<Url>> for AriesMessage {
    fn from(value: PairwiseInvitation<Url>) -> Self {
        let interm = Invitation::from(value);
        let interm = Connection::from(interm);
        AriesMessage::from(interm)
    }
}

impl From<PairwiseInvitation<String>> for AriesMessage {
    fn from(value: PairwiseInvitation<String>) -> Self {
        let interm = Invitation::from(value);
        let interm = Connection::from(interm);
        AriesMessage::from(interm)
    }
}
