use serde::{Deserialize, Serialize};

use super::CredentialPreview;
use crate::{
    decorators::{attachment::Attachment, thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type OfferCredential = MsgParts<OfferCredentialContent, OfferCredentialDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct OfferCredentialContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub credential_preview: CredentialPreview,
    #[serde(rename = "offers~attach")]
    pub offers_attach: Vec<Attachment>,
}

impl OfferCredentialContent {
    pub fn new(credential_preview: CredentialPreview, offers_attach: Vec<Attachment>) -> Self {
        Self {
            comment: None,
            credential_preview,
            offers_attach,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct OfferCredentialDecorators {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
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
        decorators::{
            attachment::tests::make_extended_attachment, thread::tests::make_extended_thread,
            timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_fields::protocols::cred_issuance::CredentialAttr,
        msg_types::cred_issuance::CredentialIssuanceTypeV1_0,
    };

    #[test]
    fn test_minimal_offer_cred() {
        let attribute = CredentialAttr::new("test_attribute_name".to_owned(), "test_attribute_value".to_owned());
        let preview = CredentialPreview::new(vec![attribute]);
        let content = OfferCredentialContent::new(preview, vec![make_extended_attachment()]);

        let decorators = OfferCredentialDecorators::default();

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
        let attribute = CredentialAttr::new("test_attribute_name".to_owned(), "test_attribute_value".to_owned());
        let preview = CredentialPreview::new(vec![attribute]);
        let mut content = OfferCredentialContent::new(preview, vec![make_extended_attachment()]);
        content.comment = Some("test_comment".to_owned());

        let mut decorators = OfferCredentialDecorators::default();
        decorators.thread = Some(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "offers~attach": {
                "field1": "value1",
                "field2": "value2",
            },
            "credential_preview": {
                "field1": "value1",
                "field2": "value2",
            },
            "comment": {
                "field1": "value1",
            },
            "~thread": {
                "field_1": "value1",
                "field_1": "value1"
            },
            "~timing": {
                "field1": "value1",
                "field2": "value2",
            }
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV1_0::OfferCredential,
            expected,
        );
    }
}
