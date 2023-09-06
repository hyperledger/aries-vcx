use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use url::Url;

pub type PairwiseInvitationContent = PwInvitationContent<Url>;
pub type PairwiseDidInvitationContent = PwInvitationContent<String>;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct PwInvitationContent<T> {
    pub label: String,
    pub recipient_keys: Vec<String>,
    #[builder(default)]
    #[serde(default)]
    pub routing_keys: Vec<String>,
    pub service_endpoint: T,
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
    fn test_minimal_conn_invite_pw() {
        let content = PairwiseInvitationContent::builder()
            .label("test_pw_invite_label".to_owned())
            .recipient_keys(vec!["test_recipient_key".to_owned()])
            .service_endpoint(Url::parse("https://dummy.dummy/dummy").unwrap())
            .build();

        let decorators = InvitationDecorators::default();

        let expected = json!({
            "label": content.label,
            "recipientKeys": content.recipient_keys,
            "routingKeys": content.routing_keys,
            "serviceEndpoint": content.service_endpoint,
        });

        test_utils::test_msg(
            InvitationContent::Pairwise(content),
            decorators,
            ConnectionTypeV1_0::Invitation,
            expected,
        );
    }

    #[test]
    fn test_extended_conn_invite_pw() {
        let content = PairwiseInvitationContent::builder()
            .label("test_pw_invite_label".to_owned())
            .recipient_keys(vec!["test_recipient_key".to_owned()])
            .routing_keys(vec!["test_routing_key".to_owned()])
            .service_endpoint(Url::parse("https://dummy.dummy/dummy").unwrap())
            .build();

        let mut decorators = InvitationDecorators::default();
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "label": content.label,
            "recipientKeys": content.recipient_keys,
            "routingKeys": content.routing_keys,
            "serviceEndpoint": content.service_endpoint,
            "~timing": decorators.timing
        });

        test_utils::test_msg(
            InvitationContent::Pairwise(content),
            decorators,
            ConnectionTypeV1_0::Invitation,
            expected,
        );
    }

    #[test]
    fn test_minimal_conn_invite_pw_did() {
        let content = PairwiseDidInvitationContent::builder()
            .label("test_pw_invite_label".to_owned())
            .recipient_keys(vec!["test_recipient_key".to_owned()])
            .service_endpoint("test_conn_invite_pw_did".to_owned())
            .build();

        let decorators = InvitationDecorators::default();

        let expected = json!({
            "label": content.label,
            "recipientKeys": content.recipient_keys,
            "routingKeys": content.routing_keys,
            "serviceEndpoint": content.service_endpoint,
        });

        test_utils::test_msg(
            InvitationContent::PairwiseDID(content),
            decorators,
            ConnectionTypeV1_0::Invitation,
            expected,
        );
    }

    #[test]
    fn test_extended_conn_invite_pw_did() {
        let content = PairwiseDidInvitationContent::builder()
            .label("test_pw_invite_label".to_owned())
            .recipient_keys(vec!["test_recipient_key".to_owned()])
            .routing_keys(vec!["test_routing_key".to_owned()])
            .service_endpoint("test_conn_invite_pw_did".to_owned())
            .build();

        let mut decorators = InvitationDecorators::default();
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "label": content.label,
            "recipientKeys": content.recipient_keys,
            "routingKeys": content.routing_keys,
            "serviceEndpoint": content.service_endpoint,
            "~timing": decorators.timing
        });

        test_utils::test_msg(
            InvitationContent::PairwiseDID(content),
            decorators,
            ConnectionTypeV1_0::Invitation,
            expected,
        );
    }
}
