use std::str::FromStr;

use messages_macros::MessageContent;
use serde::{de::Error, Deserialize, Serialize};

use crate::{
    composite_message::Message,
    decorators::{PleaseAck, Thread, Timing},
    msg_types::{
        types::{
            connection::{Connection, ConnectionV1, ConnectionV1_0Kind},
            traits::MessageKind,
        },
        MessageType, Protocol,
    },
};

use crate::protocols::traits::ConcreteMessage;

pub type Response = Message<ResponseContent, ResponseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Copy, Clone, Debug, Default)]
struct SigEd25519Sha512Single;

impl Serialize for SigEd25519Sha512Single {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let protocol = Protocol::from(ConnectionV1_0Kind::parent());
        format_args!("{protocol}/{}", ConnectionV1_0Kind::Ed25519Sha512Single.as_ref()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SigEd25519Sha512Single {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let msg_type = MessageType::deserialize(deserializer)?;

        if let Protocol::Connection(Connection::V1(ConnectionV1::V1_0(_))) = msg_type.protocol {
            if let Ok(ConnectionV1_0Kind::Ed25519Sha512Single) = ConnectionV1_0Kind::from_str(msg_type.kind) {
                return Ok(SigEd25519Sha512Single);
            }
        }

        let kind = ConnectionV1_0Kind::Ed25519Sha512Single;
        Err(D::Error::custom(format!("message kind is not {}", kind.as_ref())))
    }
}
