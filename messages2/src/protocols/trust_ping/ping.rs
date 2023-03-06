use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::Message,
    decorators::{Thread, Timing},
    message_type::message_family::trust_ping::TrustPingV1_0,
    protocols::traits::MessageKind,
};

pub type Ping = Message<PingContent, PingDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "TrustPingV1_0::PingResponse")]
pub struct PingContent {
    #[serde(default)]
    pub response_requested: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PingDecorators {
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
