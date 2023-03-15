use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Attachment, PleaseAck, Thread, Timing},
    msg_types::types::cred_issuance::CredentialIssuanceV1_0Kind,
    Message,
};

pub type IssueCredential = Message<IssueCredentialContent, IssueCredentialDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "CredentialIssuanceV1_0Kind::IssueCredential")]
pub struct IssueCredentialContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "credentials~attach")]
    pub credentials_attach: Vec<Attachment>,
}

impl IssueCredentialContent {
    pub fn new(credentials_attach: Vec<Attachment>) -> Self {
        Self {
            comment: None,
            credentials_attach,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct IssueCredentialDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

impl IssueCredentialDecorators {
    pub fn new(thread: Thread) -> Self {
        Self {
            thread,
            please_ack: None,
            timing: None,
        }
    }
}
