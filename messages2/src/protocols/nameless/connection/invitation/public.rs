use serde::{Deserialize, Serialize};

use crate::message::Message;

pub type PublicInvitation = Message<PublicInvitationContent>;

/// Represents a public invitation.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PublicInvitationContent {
    pub label: String,
    pub did: String,
}

impl PublicInvitationContent {
    pub fn new(label: String, did: String) -> Self {
        Self { label, did }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        misc::{NoDecorators, test_utils},
        protocols::nameless::connection::invitation::Invitation,
    };

    #[test]
    fn test_minimal_conn_invite_public() {
        let content = PublicInvitationContent::new("test_label".to_owned(), "test_did".to_owned());

        let expected = json!({
            "label": content.label,
            "did": content.did
        });

        test_utils::test_msg::<Invitation, _, _>(content, NoDecorators, expected);
    }
}
