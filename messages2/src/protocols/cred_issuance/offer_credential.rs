use messages_macros::Message;
use serde::{Deserialize, Serialize};

use crate::protocols::traits::ConcreteMessage;
use crate::{
    decorators::{Attachment, Thread, Timing},
    message_type::message_family::cred_issuance::CredentialIssuanceV1_0,
};

use super::CredentialPreview;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "CredentialIssuanceV1_0::OfferCredential")]
pub struct OfferCredential {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub credential_preview: CredentialPreview,
    #[serde(rename = "offers~attach")]
    pub offers_attach: Vec<Attachment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
