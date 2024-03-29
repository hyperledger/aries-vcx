use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::CredentialPreviewV1;
use crate::{
    decorators::{attachment::Attachment, thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type OfferCredentialV1 = MsgParts<OfferCredentialV1Content, OfferCredentialV1Decorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct OfferCredentialV1Content {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub credential_preview: CredentialPreviewV1,
    #[serde(rename = "offers~attach")]
    pub offers_attach: Vec<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct OfferCredentialV1Decorators {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{
            attachment::tests::make_extended_attachment, thread::tests::make_extended_thread,
            timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_fields::protocols::cred_issuance::v1::CredentialAttr,
        msg_types::cred_issuance::CredentialIssuanceTypeV1_0,
    };

    #[test]
    fn test_minimal_offer_cred() {
        let attribute = CredentialAttr::builder()
            .name("test_attribute_name".to_owned())
            .value("test_attribute_value".to_owned())
            .build();

        let preview = CredentialPreviewV1::new(vec![attribute]);
        let content = OfferCredentialV1Content::builder()
            .credential_preview(preview)
            .offers_attach(vec![make_extended_attachment()])
            .build();

        let decorators = OfferCredentialV1Decorators::default();

        let expected = json!({
            "offers~attach": content.offers_attach,
            "credential_preview": content.credential_preview,
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV1_0::OfferCredential,
            expected,
        );
    }

    #[test]
    fn test_extended_offer_cred() {
        let attribute = CredentialAttr::builder()
            .name("test_attribute_name".to_owned())
            .value("test_attribute_value".to_owned())
            .build();

        let preview = CredentialPreviewV1::new(vec![attribute]);
        let content = OfferCredentialV1Content::builder()
            .credential_preview(preview)
            .offers_attach(vec![make_extended_attachment()])
            .comment("test_comment".to_owned())
            .build();

        let decorators = OfferCredentialV1Decorators::builder()
            .thread(make_extended_thread())
            .timing(make_extended_timing())
            .build();

        let expected = json!({
            "offers~attach": content.offers_attach,
            "credential_preview": content.credential_preview,
            "comment": content.comment,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV1_0::OfferCredential,
            expected,
        );
    }
}
