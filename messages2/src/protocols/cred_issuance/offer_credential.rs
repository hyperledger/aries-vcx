use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};


use crate::Message;
use crate::{
    decorators::{Attachment, Thread, Timing},
    msg_types::types::cred_issuance::CredentialIssuanceV1_0Kind,
};

use super::CredentialPreview;

pub type OfferCredential = Message<OfferCredentialContent, OfferCredentialDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
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

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct OfferCredentialDecorators {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
