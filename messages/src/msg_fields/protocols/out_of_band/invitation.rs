use diddoc_legacy::aries::service::AriesService;
use serde::{Deserialize, Serialize};
use shared::maybe_known::MaybeKnown;
use typed_builder::TypedBuilder;

use super::OobGoalCode;
use crate::{
    decorators::{attachment::Attachment, timing::Timing},
    misc::MimeType,
    msg_parts::MsgParts,
    msg_types::Protocol,
};

pub type Invitation = MsgParts<InvitationContent, InvitationDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct InvitationContent {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<MaybeKnown<OobGoalCode>>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept: Option<Vec<MimeType>>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handshake_protocols: Option<Vec<MaybeKnown<Protocol>>>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "requests~attach")]
    pub requests_attach: Option<Vec<Attachment>>,
    pub services: Vec<OobService>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct InvitationDecorators {
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum OobService {
    AriesService(AriesService),
    Did(String),
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{
            attachment::tests::make_extended_attachment, timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_types::{out_of_band::OutOfBandTypeV1_1, protocols::connection::ConnectionTypeV1},
    };

    #[test]
    fn test_minimal_oob_invitation() {
        let content = InvitationContent::builder()
            .services(vec![OobService::Did("test_service_did".to_owned())])
            .build();

        let decorators = InvitationDecorators::default();

        let expected = json!({
            "services": content.services,
        });

        test_utils::test_msg(content, decorators, OutOfBandTypeV1_1::Invitation, expected);
    }

    #[test]
    fn test_extended_oob_invitation() {
        let content = InvitationContent::builder()
            .services(vec![OobService::Did("test_service_did".to_owned())])
            .requests_attach(vec![make_extended_attachment()])
            .label("test_label".to_owned())
            .goal_code(MaybeKnown::Known(OobGoalCode::P2PMessaging))
            .goal("test_oob_goal".to_owned())
            .accept(vec![MimeType::Json, MimeType::Plain])
            .handshake_protocols(vec![MaybeKnown::Known(ConnectionTypeV1::new_v1_0().into())])
            .build();

        let decorators = InvitationDecorators::builder()
            .timing(make_extended_timing())
            .build();

        let expected = json!({
            "label": content.label,
            "goal_code": content.goal_code,
            "goal": content.goal,
            "accept": content.accept,
            "handshake_protocols": content.handshake_protocols,
            "services": content.services,
            "requests~attach": content.requests_attach,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, OutOfBandTypeV1_1::Invitation, expected);
    }
}
