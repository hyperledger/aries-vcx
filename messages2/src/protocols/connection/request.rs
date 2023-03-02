use diddoc::aries::diddoc::AriesDidDoc;
use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Thread, Timing},
    message_type::message_family::connection::ConnectionV1_0,
};

use crate::protocols::traits::MessageKind;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "ConnectionV1_0::Request")]
pub struct Request {
    pub label: String,
    pub connection: ConnectionData,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConnectionData {
    #[serde(rename = "DID")]
    pub did: String,
    #[serde(rename = "DIDDoc")]
    pub did_doc: AriesDidDoc,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestDecorators {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
