use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::message_type::message_family::routing::{Routing, RoutingV1, RoutingV1_0};

use super::traits::MessageKind;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "Routing::V1(RoutingV1::V1_0(RoutingV1_0::Forward))")]
pub struct Forward {
    pub to: String,
    #[serde(rename = "msg")]
    pub msg: Box<RawValue>,
}

impl Forward {
    pub fn new(to: String, msg: Box<RawValue>) -> Self {
        Self { to, msg }
    }
}
