use serde::{Deserialize, Serialize};

use crate::msg_parts::MsgParts;

pub type PublicInvitation = MsgParts<PublicInvitationContent>;

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
    use crate::{misc::test_utils, msg_types::connection::ConnectionTypeV1_0};
    use shared_vcx::misc::no_decorators::NoDecorators;

    #[test]
    fn test_minimal_conn_invite_public() {
        let content = PublicInvitationContent::new("test_label".to_owned(), "test_did".to_owned());

        let expected = json!({
            "label": content.label,
            "did": content.did
        });

        test_utils::test_msg(content, NoDecorators, ConnectionTypeV1_0::Invitation, expected);
    }
}
