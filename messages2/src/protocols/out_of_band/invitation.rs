use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use super::OobGoalCode;
use crate::{
    decorators::{attachment::Attachment, timing::Timing},
    misc::mime_type::MimeType,
    msg_types::{types::out_of_band::OutOfBandV1_1Kind, Protocol},
    protocols::common::service::Service,
    Message,
};

pub type Invitation = Message<InvitationContent, InvitationDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "OutOfBandV1_1Kind::Invitation")]
pub struct InvitationContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<OobGoalCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept: Option<Vec<MimeType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handshake_protocols: Option<Vec<Protocol>>,
    pub services: Vec<Service>,
    #[serde(rename = "requests~attach")]
    pub requests_attach: Vec<Attachment>,
}

impl InvitationContent {
    pub fn new(services: Vec<Service>, requests_attach: Vec<Attachment>) -> Self {
        Self {
            label: None,
            goal_code: None,
            goal: None,
            accept: None,
            handshake_protocols: None,
            services,
            requests_attach,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct InvitationDecorators {
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
        decorators::{attachment::tests::make_extended_attachment, timing::tests::make_extended_timing},
        misc::test_utils,
        msg_types::types::connection::ConnectionV1,
    };

    #[test]
    fn test_minimal_oob_invitation() {
        let msg_type = test_utils::build_msg_type::<InvitationContent>();

        let content = InvitationContent::new(
            vec![Service::Did("test_service_did".to_owned())],
            vec![make_extended_attachment()],
        );

        let decorators = InvitationDecorators::default();

        let json = json!({
            "@type": msg_type,
            "services": content.services,
            "requests~attach": content.requests_attach,
        });

        test_utils::test_msg(content, decorators, json);
    }

    #[test]
    fn test_extensive_oob_invitation() {
        let msg_type = test_utils::build_msg_type::<InvitationContent>();

        let mut content = InvitationContent::new(
            vec![Service::Did("test_service_did".to_owned())],
            vec![make_extended_attachment()],
        );

        content.label = Some("test_label".to_owned());
        content.goal_code = Some(OobGoalCode::P2PMessaging);
        content.goal = Some("test_oob_goal".to_owned());
        content.accept = Some(vec![MimeType::Json, MimeType::Plain]);
        content.handshake_protocols = Some(vec![ConnectionV1::new_v1_0().into()]);

        let mut decorators = InvitationDecorators::default();
        decorators.timing = Some(make_extended_timing());

        let json = json!({
            "@type": msg_type,
            "label": content.label,
            "goal_code": content.goal_code,
            "goal": content.goal,
            "accept": content.accept,
            "handshake_protocols": content.handshake_protocols,
            "services": content.services,
            "requests~attach": content.requests_attach,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, json);
    }
}
