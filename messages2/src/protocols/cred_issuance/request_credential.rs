use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Attachment, Thread, Timing},
    msg_types::types::cred_issuance::CredentialIssuanceV1_0Kind,
    Message,
};

pub type RequestCredential = Message<RequestCredentialContent, RequestCredentialDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "CredentialIssuanceV1_0Kind::RequestCredential")]
pub struct RequestCredentialContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "requests~attach")]
    pub requests_attach: Vec<Attachment>,
}

impl RequestCredentialContent {
    pub fn new(requests_attach: Vec<Attachment>) -> Self {
        Self {
            comment: None,
            requests_attach,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct RequestCredentialDecorators {
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
