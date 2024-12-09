use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::CredentialPreviewV2;
use crate::{
    decorators::{attachment::Attachment, thread::Thread, timing::Timing},
    msg_fields::protocols::common::attachment_format_specifier::AttachmentFormatSpecifier,
    msg_parts::MsgParts,
};

pub type ProposeCredentialV2 = MsgParts<ProposeCredentialV2Content, ProposeCredentialV2Decorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct ProposeCredentialV2Content {
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<String>, // TODO - spec does not specify what goal codes to use..
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_preview: Option<CredentialPreviewV2>,
    pub formats: Vec<AttachmentFormatSpecifier<ProposeCredentialAttachmentFormatType>>,
    #[serde(rename = "filters~attach")]
    pub filters_attach: Vec<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct ProposeCredentialV2Decorators {
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[builder(default)]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum ProposeCredentialAttachmentFormatType {
    #[serde(rename = "dif/credential-manifest@v1.0")]
    DifCredentialManifest1_0,
    #[serde(rename = "aries/ld-proof-vc-detail@v1.0")]
    AriesLdProofVcDetail1_0,
    #[serde(rename = "anoncreds/credential-filter@v1.0")]
    AnoncredCredentialFilter1_0,
    #[serde(rename = "hlindy/cred-filter@v2.0")]
    HyperledgerIndyCredentialFilter2_0,
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
            timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_fields::protocols::cred_issuance::common::CredentialAttr,
        msg_types::cred_issuance::CredentialIssuanceTypeV2_0,
    };

    #[test]
    fn test_minimal_propose_cred() {
        let content = ProposeCredentialV2Content::builder()
            .formats(vec![AttachmentFormatSpecifier {
                attach_id: String::from("1"),
                format: MaybeKnown::Known(
                    ProposeCredentialAttachmentFormatType::HyperledgerIndyCredentialFilter2_0,
                ),
            }])
            .filters_attach(vec![make_extended_attachment()])
            .build();

        let decorators = ProposeCredentialV2Decorators::default();

        let expected = json!({
            "formats": content.formats,
            "filters~attach": content.filters_attach,
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV2_0::ProposeCredential,
            expected,
        );
    }

    #[test]
    fn test_extended_propose_cred() {
        let attribute = CredentialAttr::builder()
            .name("test_attribute_name".to_owned())
            .value("test_attribute_value".to_owned())
            .build();
        let preview = CredentialPreviewV2::new(vec![attribute]);
        let content = ProposeCredentialV2Content::builder()
            .credential_preview(Some(preview))
            .formats(vec![AttachmentFormatSpecifier {
                attach_id: String::from("1"),
                format: MaybeKnown::Known(
                    ProposeCredentialAttachmentFormatType::HyperledgerIndyCredentialFilter2_0,
                ),
            }])
            .filters_attach(vec![make_extended_attachment()])
            .comment(Some("test_comment".to_owned()))
            .goal_code(Some("goal.goal".to_owned()))
            .build();

        let decorators = ProposeCredentialV2Decorators::builder()
            .thread(Some(make_extended_thread()))
            .timing(Some(make_extended_timing()))
            .build();

        let expected = json!({
            "credential_preview": content.credential_preview,
            "formats": content.formats,
            "filters~attach": content.filters_attach,
            "comment": content.comment,
            "goal_code": content.goal_code,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV2_0::ProposeCredential,
            expected,
        );
    }
}
