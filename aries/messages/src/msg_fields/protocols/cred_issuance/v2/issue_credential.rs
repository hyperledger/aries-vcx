use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{attachment::Attachment, please_ack::PleaseAck, thread::Thread, timing::Timing},
    msg_fields::protocols::common::attachment_format_specifier::AttachmentFormatSpecifier,
    msg_parts::MsgParts,
};

pub type IssueCredentialV2 = MsgParts<IssueCredentialV2Content, IssueCredentialV2Decorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct IssueCredentialV2Content {
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<String>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement_id: Option<String>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub formats: Vec<AttachmentFormatSpecifier<IssueCredentialAttachmentFormatType>>,
    #[serde(rename = "credentials~attach")]
    pub credentials_attach: Vec<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct IssueCredentialV2Decorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[builder(default)]
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
    #[builder(default)]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum IssueCredentialAttachmentFormatType {
    #[serde(rename = "aries/ld-proof-vc@v1.0")]
    AriesLdProofVc1_0,
    #[serde(rename = "anoncreds/credential@v1.0")]
    AnoncredsCredential1_0,
    #[serde(rename = "hlindy/cred@v2.0")]
    HyperledgerIndyCredential2_0,
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;
    use shared::maybe_known::MaybeKnown;

    use super::*;
    use crate::{
        decorators::{
            attachment::tests::make_extended_attachment,
            please_ack::tests::make_minimal_please_ack, thread::tests::make_extended_thread,
            timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_types::cred_issuance::CredentialIssuanceTypeV2_0,
    };

    #[test]
    fn test_minimal_issue_cred() {
        let content = IssueCredentialV2Content::builder()
            .formats(vec![AttachmentFormatSpecifier {
                attach_id: "1".to_owned(),
                format: MaybeKnown::Known(
                    IssueCredentialAttachmentFormatType::HyperledgerIndyCredential2_0,
                ),
            }])
            .credentials_attach(vec![make_extended_attachment()])
            .build();

        let decorators = IssueCredentialV2Decorators::builder()
            .thread(make_extended_thread())
            .build();

        let expected = json!({
            "formats": content.formats,
            "credentials~attach": content.credentials_attach,
            "~thread": decorators.thread
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV2_0::IssueCredential,
            expected,
        );
    }

    #[test]
    fn test_extended_issue_cred() {
        let content = IssueCredentialV2Content::builder()
            .formats(vec![AttachmentFormatSpecifier {
                attach_id: "1".to_owned(),
                format: shared::maybe_known::MaybeKnown::Known(
                    IssueCredentialAttachmentFormatType::HyperledgerIndyCredential2_0,
                ),
            }])
            .credentials_attach(vec![make_extended_attachment()])
            .goal_code(Some("goal.goal".to_owned()))
            .replacement_id(Some("replacement-123".to_owned()))
            .comment(Some("test_comment".to_owned()))
            .build();

        let decorators = IssueCredentialV2Decorators::builder()
            .thread(make_extended_thread())
            .timing(Some(make_extended_timing()))
            .please_ack(Some(make_minimal_please_ack()))
            .build();

        let expected = json!({
            "formats": content.formats,
            "credentials~attach": content.credentials_attach,
            "goal_code": content.goal_code,
            "replacement_id": content.replacement_id,
            "comment": content.comment,
            "~thread": decorators.thread,
            "~timing": decorators.timing,
            "~please_ack": decorators.please_ack
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV2_0::IssueCredential,
            expected,
        );
    }
}
