use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::Message,
    decorators::{PleaseAck, Thread, Timing},
    message_type::{message_protocol::connection::ConnectionV1_0Kind, MessageFamily},
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

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
// #[serde(into = "MessageType", try_from = "MessageType")]
struct SigEd25519Sha512Single;

impl From<SigEd25519Sha512Single> for ConnectionV1_0Kind {
    fn from(_value: SigEd25519Sha512Single) -> Self {
        ConnectionV1_0Kind::Ed25519Sha512Single
    }
}

impl TryFrom<ConnectionV1_0Kind> for SigEd25519Sha512Single {
    type Error = &'static str;

    fn try_from(value: ConnectionV1_0Kind) -> Result<Self, Self::Error> {
        match value {
            ConnectionV1_0Kind::Ed25519Sha512Single => Ok(Self),
            _ => Err("message kind is not \"ed25519Sha512_single\""),
        }
    }
}

// impl TryFrom<MessageType> for SigEd25519Sha512Single {
//     type Error = &'static str;

//     fn try_from(value: MessageType) -> Result<Self, Self::Error> {
//         let interm = MessageFamily::from(value);
//         SigEd25519Sha512Single::try_from(interm)
//     }
// }
