use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Thread, Timing},
    msg_types::types::trust_ping::TrustPingV1_0Kind,
    protocols::traits::ConcreteMessage, Message,
};

pub type PingResponse = Message<PingResponseContent, PingResponseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, Default)]
#[message(kind = "TrustPingV1_0Kind::PingResponse")]
pub struct PingResponseContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PingResponseDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

impl PingResponseDecorators {
    pub fn new(thread: Thread) -> Self {
        Self { thread, timing: None }
    }
}
