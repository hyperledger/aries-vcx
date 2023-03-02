use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Attachment, Thread, Timing},
    message_type::message_family::cred_issuance::CredentialIssuanceV1_0,
    protocols::traits::MessageKind,
};

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "CredentialIssuanceV1_0::RequestCredential")]
pub struct RequestCredential {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "requests~attach")]
    pub requests_attach: Vec<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestCredentialDecorators {
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
