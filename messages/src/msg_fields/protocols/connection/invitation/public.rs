use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct PublicInvitationContent {
    pub label: String,
    pub did: String,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::timing::tests::make_extended_timing,
        misc::test_utils,
        msg_fields::protocols::connection::invitation::{InvitationContent, InvitationDecorators},
        msg_types::connection::ConnectionTypeV1_0,
    };

    #[test]
    fn test_minimal_conn_invite_public() {
        let content = PublicInvitationContent::builder()
            .label("test_label".to_owned())
            .did("test_did".to_owned())
            .build();

        let expected = json!({
            "label": content.label,
            "did": content.did
        });

        let decorators = InvitationDecorators::default();

        test_utils::test_msg(
            InvitationContent::Public(content),
            decorators,
            ConnectionTypeV1_0::Invitation,
            expected,
        );
    }

    #[test]
    fn test_extended_conn_invite_public() {
        let content = PublicInvitationContent::builder()
            .label("test_label".to_owned())
            .did("test_did".to_owned())
            .build();

        let expected = json!({
            "label": content.label,
            "did": content.did
        });

        let mut decorators = InvitationDecorators::default();
        decorators.timing = Some(make_extended_timing());

        test_utils::test_msg(
            InvitationContent::Public(content),
            decorators,
            ConnectionTypeV1_0::Invitation,
            expected,
        );
    }
}
