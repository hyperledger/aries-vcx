use std::str::FromStr;

use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{please_ack::PleaseAck, thread::Thread, timing::Timing},
    message::Message,
    msg_types::{
        traits::MessageKind,
        types::connection::{Connection, ConnectionV1, ConnectionV1_0},
        MessageType, Protocol,
    },
};

pub type Response = Message<ResponseContent, ResponseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "ConnectionV1_0::Response")]
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

impl<'a> From<&'a SigEd25519Sha512Single> for ConnectionV1_0 {
    fn from(_value: &'a SigEd25519Sha512Single) -> Self {
        ConnectionV1_0::Ed25519Sha512Single
    }
}

impl<'a> TryFrom<MessageType<'a>> for SigEd25519Sha512Single {
    type Error = String;

    fn try_from(value: MessageType<'a>) -> Result<Self, Self::Error> {
        if let Protocol::Connection(Connection::V1(ConnectionV1::V1_0(_))) = value.protocol {
            if let Ok(ConnectionV1_0::Ed25519Sha512Single) = ConnectionV1_0::from_str(value.kind) {
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
        let protocol = Protocol::from(ConnectionV1_0::parent());
        let kind = ConnectionV1_0::from(self);
        format_args!("{protocol}/{}", kind.as_ref()).serialize(serializer)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{
            please_ack::tests::make_minimal_please_ack, thread::tests::make_extended_thread,
            timing::tests::make_extended_timing,
        },
        misc::test_utils,
    };

    #[test]
    fn test_minimal_conn_response() {
        let conn_sig = ConnectionSignature::new(
            "test_signature".to_owned(),
            "test_sig_data".to_owned(),
            "test_signer".to_owned(),
        );

        let content = ResponseContent::new(conn_sig);

        let decorators = ResponseDecorators::new(make_extended_thread());

        let json = json!({
            "connection~sig": content.connection_sig,
            "~thread": decorators.thread
        });

        test_utils::test_msg::<ResponseContent, _, _>(content, decorators, json);
    }

    #[test]
    fn test_extended_conn_response() {
        let conn_sig = ConnectionSignature::new(
            "test_signature".to_owned(),
            "test_sig_data".to_owned(),
            "test_signer".to_owned(),
        );

        let content = ResponseContent::new(conn_sig);

        let mut decorators = ResponseDecorators::new(make_extended_thread());
        decorators.timing = Some(make_extended_timing());
        decorators.please_ack = Some(make_minimal_please_ack());

        let json = json!({
            "connection~sig": content.connection_sig,
            "~thread": decorators.thread,
            "~timing": decorators.timing,
            "~please_ack": decorators.please_ack
        });

        test_utils::test_msg::<ResponseContent, _, _>(content, decorators, json);
    }
}
