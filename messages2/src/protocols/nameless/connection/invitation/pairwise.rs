use serde::{Deserialize, Serialize};
use url::Url;

use crate::{decorators::timing::Timing, message::Message};

pub type PairwiseInvitation = Message<PairwiseInvitationContent<Url>, PwInvitationDecorators>;
pub type PairwiseDidInvitation = Message<PairwiseInvitationContent<String>, PwInvitationDecorators>;

/// Wrapper that represents a pairwise invitation.
// The wrapping is used so that we expose certain types as an abstraction
// over our internal types.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PairwiseInvitationContent<T> {
    pub label: String,
    pub recipient_keys: Vec<String>,
    #[serde(default)]
    pub routing_keys: Vec<String>,
    pub service_endpoint: T,
}

impl<T> PairwiseInvitationContent<T> {
    pub fn new(label: String, recipient_keys: Vec<String>, routing_keys: Vec<String>, service_endpoint: T) -> Self {
        Self {
            label,
            recipient_keys,
            routing_keys,
            service_endpoint,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct PwInvitationDecorators {
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::timing::tests::make_extended_timing, misc::test_utils,
        protocols::nameless::connection::invitation::Invitation,
    };

    #[test]
    fn test_minimal_conn_invite_pw() {
        let content = PairwiseInvitationContent::new(
            "test_pw_invite_label".to_owned(),
            vec!["test_recipient_key".to_owned()],
            vec![],
            Url::parse("https://dummy.dummy/dummy").unwrap(),
        );

        let decorators = PwInvitationDecorators::default();

        let expected = json!({
            "label": content.label,
            "recipientKeys": content.recipient_keys,
            "routingKeys": content.routing_keys,
            "serviceEndpoint": content.service_endpoint,
        });

        test_utils::test_msg::<Invitation, _, _>(content, decorators, expected);
    }

    #[test]
    fn test_extended_conn_invite_pw() {
        let content = PairwiseInvitationContent::new(
            "test_pw_invite_label".to_owned(),
            vec!["test_recipient_key".to_owned()],
            vec!["test_routing_key".to_owned()],
            Url::parse("https://dummy.dummy/dummy").unwrap(),
        );

        let mut decorators = PwInvitationDecorators::default();
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "label": content.label,
            "recipientKeys": content.recipient_keys,
            "routingKeys": content.routing_keys,
            "serviceEndpoint": content.service_endpoint,
            "~timing": decorators.timing
        });

        test_utils::test_msg::<Invitation, _, _>(content, decorators, expected);
    }

    #[test]
    fn test_minimal_conn_invite_pw_did() {
        let content = PairwiseInvitationContent::new(
            "test_pw_invite_label".to_owned(),
            vec!["test_recipient_key".to_owned()],
            vec![],
            "test_conn_invite_pw_did".to_owned(),
        );

        let decorators = PwInvitationDecorators::default();

        let expected = json!({
            "label": content.label,
            "recipientKeys": content.recipient_keys,
            "routingKeys": content.routing_keys,
            "serviceEndpoint": content.service_endpoint,
        });

        test_utils::test_msg::<Invitation, _, _>(content, decorators, expected);
    }

    #[test]
    fn test_extended_conn_invite_pw_did() {
        let content = PairwiseInvitationContent::new(
            "test_pw_invite_label".to_owned(),
            vec!["test_recipient_key".to_owned()],
            vec!["test_routing_key".to_owned()],
            "test_conn_invite_pw_did".to_owned(),
        );

        let mut decorators = PwInvitationDecorators::default();
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "label": content.label,
            "recipientKeys": content.recipient_keys,
            "routingKeys": content.routing_keys,
            "serviceEndpoint": content.service_endpoint,
            "~timing": decorators.timing
        });

        test_utils::test_msg::<Invitation, _, _>(content, decorators, expected);
    }
}
