use std::str::FromStr;

use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{please_ack::PleaseAck, thread::Thread, timing::Timing},
    msg_types::{
        types::{
            connection::{Connection, ConnectionV1, ConnectionV1_0Kind},
            traits::MessageKind,
        },
        MessageType, Protocol,
    },
    Message,
};

pub type Response = Message<ResponseContent, ResponseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "ConnectionV1_0Kind::Response")]
pub struct ResponseContent {
    #[serde(rename = "connection~sig")]
    pub connection_sig: ConnectionSignature,
}

impl ResponseContent {
    pub fn new(connection_sig: ConnectionSignature) -> Self {
        Self { connection_sig }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ConnectionSignature {
    #[serde(rename = "@type")]
    msg_type: SigEd25519Sha512Single,
    pub signature: String,
    pub sig_data: String,
    pub signer: String,
}

impl ConnectionSignature {
    pub fn new(signature: String, sig_data: String, signer: String) -> Self {
        Self {
            msg_type: SigEd25519Sha512Single,
            signature,
            sig_data,
            signer,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ResponseDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

impl ResponseDecorators {
    pub fn new(thread: Thread) -> Self {
        Self {
            thread,
            please_ack: None,
            timing: None,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Default, PartialEq)]
#[serde(try_from = "MessageType")]
struct SigEd25519Sha512Single;

impl<'a> From<&'a SigEd25519Sha512Single> for ConnectionV1_0Kind {
    fn from(_value: &'a SigEd25519Sha512Single) -> Self {
        ConnectionV1_0Kind::Ed25519Sha512Single
    }
}

impl<'a> TryFrom<MessageType<'a>> for SigEd25519Sha512Single {
    type Error = String;

    fn try_from(value: MessageType<'a>) -> Result<Self, Self::Error> {
        if let Protocol::Connection(Connection::V1(ConnectionV1::V1_0(_))) = value.protocol {
            if let Ok(ConnectionV1_0Kind::Ed25519Sha512Single) = ConnectionV1_0Kind::from_str(value.kind) {
                return Ok(SigEd25519Sha512Single);
            }
        }

        Err(format!("message kind is not {}", value.kind))
    }
}

impl Serialize for SigEd25519Sha512Single {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let protocol = Protocol::from(ConnectionV1_0Kind::parent());
        let kind = ConnectionV1_0Kind::from(self);
        format_args!("{protocol}/{}", kind.as_ref()).serialize(serializer)
    }
}
