use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{PleaseAck, Thread, Timing},
    msg_types::types::revocation::RevocationV2_0Kind,
    Message,
};

pub type Revoke = Message<RevokeContent, RevokeDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "RevocationV2_0Kind::Revoke")]
pub struct RevokeContent {
    pub credential_id: String,
    pub revocation_format: RevocationFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl RevokeContent {
    pub fn new(credential_id: String, revocation_format: RevocationFormat) -> Self {
        Self {
            credential_id,
            revocation_format,
            comment: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RevokeDecorators {
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum RevocationFormat {
    IndyAnoncreds,
}
