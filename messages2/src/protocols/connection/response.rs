use messages_macros::Message;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Thread, Timing, PleaseAck},
    message_type::{message_family::connection::ConnectionV1_0, MessageType},
};

use crate::protocols::traits::ConcreteMessage;

use super::ConnectionData;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "ConnectionV1_0::Response")]
pub struct Response {
    #[serde(rename = "@id")]
    pub id: String,
    pub connection: ConnectionData,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct SignedResponse {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "connection~sig")]
    pub connection_sig: ConnectionSignature,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ConnectionSignature {
    #[serde(rename = "@type")]
    pub msg_type: MessageType, // FIX: Need to accommodate this
    pub signature: String,
    pub sig_data: String,
    pub signer: String,
}