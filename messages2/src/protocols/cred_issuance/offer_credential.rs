use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::protocols::traits::MessageKind;
use crate::{
    decorators::{Attachment, Thread, Timing},
    message_type::message_family::cred_issuance::CredentialIssuanceV1_0,
};

use super::CredentialPreview;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "CredentialIssuanceV1_0::OfferCredential")]
pub struct OfferCredential {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub credential_preview: CredentialPreview,
    #[serde(rename = "offers~attach")]
    pub offers_attach: Vec<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OfferCredentialDecorators {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
