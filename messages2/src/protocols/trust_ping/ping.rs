use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::Message,
    decorators::{Thread, Timing},
    msg_types::types::trust_ping::TrustPingV1_0Kind,
    protocols::traits::ConcreteMessage,
};

pub type Ping = Message<PingContent, PingDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, Default)]
#[message(kind = "TrustPingV1_0Kind::PingResponse")]
pub struct PingContent {
    #[serde(default)]
    pub response_requested: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct PingDecorators {
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
