use messages_macros::Message;
use serde::{Deserialize, Serialize};

use crate::{message_type::message_family::trust_ping::TrustPingV1_0, decorators::{Thread, Timing}, protocols::traits::ConcreteMessage};

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "TrustPingV1_0::PingResponse")]

pub struct Ping {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default)]
    pub response_requested: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}