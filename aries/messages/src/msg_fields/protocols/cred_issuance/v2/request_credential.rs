use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{attachment::Attachment, thread::Thread, timing::Timing},
    msg_fields::protocols::common::attachment_format_specifier::AttachmentFormatSpecifier,
    msg_parts::MsgParts,
};

pub type RequestCredentialV2 = MsgParts<RequestCredentialV2Content, RequestCredentialV2Decorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct RequestCredentialV2Content {
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<String>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub formats: Vec<AttachmentFormatSpecifier<RequestCredentialAttachmentFormatType>>,
    #[serde(rename = "requests~attach")]
    pub requests_attach: Vec<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct RequestCredentialV2Decorators {
    #[builder(default)]
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[builder(default)]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum RequestCredentialAttachmentFormatType {
    #[serde(rename = "dif/credential-manifest@v1.0")]
    DifCredentialManifest1_0,
    #[serde(rename = "hlindy/cred-req@v2.0")]
    HyperledgerIndyCredentialRequest2_0,
    #[serde(rename = "anoncreds/credential-request@v1.0")]
    AnoncredsCredentialRequest1_0,
    #[serde(rename = "aries/ld-proof-vc-detail@v1.0")]
    AriesLdProofVcDetail1_0,
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;
    use shared::maybe_known::MaybeKnown;

    use super::*;
    use crate::{
        decorators::{
            attachment::tests::make_extended_attachment, thread::tests::make_extended_thread,
        },
        misc::test_utils,
        msg_types::cred_issuance::CredentialIssuanceTypeV2_0,
    };

    #[test]
    fn test_minimal_request_cred() {
        let content = RequestCredentialV2Content::builder()
            .requests_attach(vec![make_extended_attachment()])
            .formats(vec![AttachmentFormatSpecifier {
                attach_id: "1".to_owned(),
                format: MaybeKnown::Known(
                    RequestCredentialAttachmentFormatType::HyperledgerIndyCredentialRequest2_0,
                ),
            }])
            .build();

        let decorators = RequestCredentialV2Decorators::default();

        let expected = json!({
            "requests~attach": content.requests_attach,
            "formats": content.formats
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV2_0::RequestCredential,
            expected,
        );
    }

    #[test]
    fn test_extended_request_cred() {
        let content = RequestCredentialV2Content::builder()
            .requests_attach(vec![make_extended_attachment()])
            .formats(vec![AttachmentFormatSpecifier {
                attach_id: "1".to_owned(),
                format: MaybeKnown::Known(
                    RequestCredentialAttachmentFormatType::HyperledgerIndyCredentialRequest2_0,
                ),
            }])
            .comment(Some("test_comment".to_owned()))
            .goal_code(Some("goal.goal".to_owned()))
            .build();

        let decorators = RequestCredentialV2Decorators::builder()
            .thread(Some(make_extended_thread()))
            .build();

        let expected = json!({
            "requests~attach": content.requests_attach,
            "formats": content.formats,
            "comment": content.comment,
            "goal_code": content.goal_code,
            "~thread": decorators.thread
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV2_0::RequestCredential,
            expected,
        );
    }
}
