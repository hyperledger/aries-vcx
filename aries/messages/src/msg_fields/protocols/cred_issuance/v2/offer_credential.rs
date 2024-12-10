use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::CredentialPreviewV2;
use crate::{
    decorators::{attachment::Attachment, thread::Thread, timing::Timing},
    msg_fields::protocols::common::attachment_format_specifier::AttachmentFormatSpecifier,
    msg_parts::MsgParts,
};

pub type OfferCredentialV2 = MsgParts<OfferCredentialV2Content, OfferCredentialV2Decorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct OfferCredentialV2Content {
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<String>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement_id: Option<String>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub credential_preview: CredentialPreviewV2,
    pub formats: Vec<AttachmentFormatSpecifier<OfferCredentialAttachmentFormatType>>,
    #[serde(rename = "offers~attach")]
    pub offers_attach: Vec<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct OfferCredentialV2Decorators {
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
pub enum OfferCredentialAttachmentFormatType {
    #[serde(rename = "dif/credential-manifest@v1.0")]
    DifCredentialManifest1_0,
    #[serde(rename = "hlindy/cred-abstract@v2.0")]
    HyperledgerIndyCredentialAbstract2_0,
    #[serde(rename = "anoncreds/credential-offer@v1.0")]
    AnoncredsCredentialOffer1_0,
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
            timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_fields::protocols::cred_issuance::common::CredentialAttr,
        msg_types::cred_issuance::CredentialIssuanceTypeV2_0,
    };

    #[test]
    fn test_minimal_offer_cred() {
        let attribute = CredentialAttr::builder()
            .name("test_attribute_name".to_owned())
            .value("test_attribute_value".to_owned())
            .build();

        let preview = CredentialPreviewV2::new(vec![attribute]);
        let content = OfferCredentialV2Content::builder()
            .credential_preview(preview)
            .formats(vec![AttachmentFormatSpecifier {
                attach_id: "1".to_owned(),
                format: MaybeKnown::Known(
                    OfferCredentialAttachmentFormatType::HyperledgerIndyCredentialAbstract2_0,
                ),
            }])
            .offers_attach(vec![make_extended_attachment()])
            .build();

        let decorators = OfferCredentialV2Decorators::default();

        let expected = json!({
            "formats": content.formats,
            "offers~attach": content.offers_attach,
            "credential_preview": content.credential_preview,
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV2_0::OfferCredential,
            expected,
        );
    }

    #[test]
    fn test_extended_offer_cred() {
        let attribute = CredentialAttr::builder()
            .name("test_attribute_name".to_owned())
            .value("test_attribute_value".to_owned())
            .build();

        let preview = CredentialPreviewV2::new(vec![attribute]);
        let content = OfferCredentialV2Content::builder()
            .credential_preview(preview)
            .formats(vec![AttachmentFormatSpecifier {
                attach_id: "1".to_owned(),
                format: MaybeKnown::Known(
                    OfferCredentialAttachmentFormatType::HyperledgerIndyCredentialAbstract2_0,
                ),
            }])
            .offers_attach(vec![make_extended_attachment()])
            .comment(Some("test_comment".to_owned()))
            .replacement_id(Some("replacement_id".to_owned()))
            .goal_code(Some("goal.goal".to_owned()))
            .build();

        let decorators = OfferCredentialV2Decorators::builder()
            .thread(Some(make_extended_thread()))
            .timing(Some(make_extended_timing()))
            .build();

        let expected = json!({
            "formats": content.formats,
            "offers~attach": content.offers_attach,
            "credential_preview": content.credential_preview,
            "comment": content.comment,
            "goal_code": content.goal_code,
            "replacement_id": content.replacement_id,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV2_0::OfferCredential,
            expected,
        );
    }
}
