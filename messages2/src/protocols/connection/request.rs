use diddoc::aries::diddoc::AriesDidDoc;
use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::Message,
    decorators::{Thread, Timing},
    message_type::message_protocol::connection::ConnectionV1_0Kind,
};

use crate::protocols::traits::ConcreteMessage;

pub type Request = Message<RequestContent, RequestDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "ConnectionV1_0Kind::Request")]
pub struct RequestContent {
    pub label: String,
    pub connection: ConnectionData,
}

impl RequestContent {
    pub fn new(label: String, connection: ConnectionData) -> Self {
        Self { label, connection }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConnectionData {
    #[serde(rename = "DID")]
    pub did: String,
    #[serde(rename = "DIDDoc")]
    pub did_doc: AriesDidDoc,
}

impl ConnectionData {
    pub fn new(did: String, did_doc: AriesDidDoc) -> Self {
        Self { did, did_doc }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct RequestDecorators {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
