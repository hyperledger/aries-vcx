use messages_macros::Message;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Thread, Timing},
    message_type::{message_family::connection::ConnectionV1_0},
};

use crate::protocols::traits::ConcreteMessage;

use super::ConnectionData;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "ConnectionV1_0::Request")]
pub struct Request {
    #[serde(rename = "@id")]
    pub id: String,
    pub label: String,
    pub connection: ConnectionData,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}