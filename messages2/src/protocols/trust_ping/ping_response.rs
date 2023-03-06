use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::Message,
    decorators::{Thread, Timing},
    message_type::message_family::trust_ping::TrustPingV1_0,
    protocols::traits::MessageKind,
};

pub type PingResponse = Message<PingResponseContent, PingResponseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "TrustPingV1_0::PingResponse")]
pub struct PingResponseContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PingResponseDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
