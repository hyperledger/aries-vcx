use messages_macros::Message;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::message_type::message_family::routing::{Routing, RoutingV1, RoutingV1_0};

use super::traits::ConcreteMessage;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "Routing::V1(RoutingV1::V1_0(RoutingV1_0::Forward))")]
pub struct Forward {
    #[serde(rename = "@id")]
    pub id: String,
    pub to: String,
    #[serde(rename = "msg")]
    pub msg: Box<RawValue>,
}
