use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use super::CredentialPreview;
use crate::{
    decorators::{attachment::Attachment, thread::Thread, timing::Timing},
    msg_types::types::cred_issuance::CredentialIssuanceV1_0Kind,
    Message,
};

pub type OfferCredential = Message<OfferCredentialContent, OfferCredentialDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "CredentialIssuanceV1_0Kind::OfferCredential")]
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
        protocols::cred_issuance::CredentialAttr,
    };

    #[test]
    fn test_minimal_offer_cred() {
        let attribute = CredentialAttr::new("test_attribute_name".to_owned(), "test_attribute_value".to_owned());
        let preview = CredentialPreview::new(vec![attribute]);
        let content = OfferCredentialContent::new(preview, vec![make_extended_attachment()]);

        let decorators = OfferCredentialDecorators::default();

        let json = json!({
            "offers~attach": content.offers_attach,
            "credential_preview": content.credential_preview,
        });

        test_utils::test_msg::<OfferCredentialContent, _, _>(content, decorators, json);
    }

    #[test]
    fn test_extensive_offer_cred() {
        let attribute = CredentialAttr::new("test_attribute_name".to_owned(), "test_attribute_value".to_owned());
        let preview = CredentialPreview::new(vec![attribute]);
        let mut content = OfferCredentialContent::new(preview, vec![make_extended_attachment()]);
        content.comment = Some("test_comment".to_owned());

        let mut decorators = OfferCredentialDecorators::default();
        decorators.thread = Some(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let json = json!({
            "offers~attach": content.offers_attach,
            "credential_preview": content.credential_preview,
            "comment": content.comment,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg::<OfferCredentialContent, _, _>(content, decorators, json);
    }
}
