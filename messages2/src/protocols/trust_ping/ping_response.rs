use messages_macros::Message;
use serde::{Deserialize, Serialize};
use transitive::TransitiveFrom;

use crate::{
    decorators::{Thread, Timing},
    message_type::message_family::trust_ping::TrustPingV1_0,
    protocols::traits::ConcreteMessage, aries_message::AriesMessage, macros::threadlike_impl,
};

use super::TrustPing;

#[derive(Clone, Debug, Deserialize, Serialize, Message, TransitiveFrom)]
#[message(kind = "TrustPingV1_0::PingResponse")]
#[transitive(into(TrustPing, AriesMessage))]
pub struct PingResponse {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

threadlike_impl!(PingResponse);