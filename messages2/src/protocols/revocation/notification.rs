use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::Message,
    decorators::{PleaseAck, Thread, Timing},
    message_type::message_family::revocation::RevocationV2_0,
    protocols::traits::MessageKind,
};

pub type Revoke = Message<RevokeContent, RevokeDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "RevocationV2_0::Revoke")]
pub struct RevokeContent {
    credential_id: String,
    revocation_format: RevocationFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RevokeDecorators {
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    please_ack: Option<PleaseAck>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    timing: Option<Timing>,
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    thread: Option<Thread>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RevocationFormat {
    #[default]
    IndyAnoncreds,
}
