use serde::{Deserialize, Serialize};
use shared_vcx::misc::utils::CowStr;
use typed_builder::TypedBuilder;

use crate::{
    decorators::{please_ack::PleaseAck, thread::Thread, timing::Timing},
    msg_parts::MsgParts,
    msg_types::{
        protocols::signature::{SignatureType, SignatureTypeV1, SignatureTypeV1_0},
        traits::MessageKind,
        MessageType, Protocol,
    },
};

pub type Response = MsgParts<ResponseContent, ResponseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct ResponseContent {
    #[serde(rename = "connection~sig")]
    pub connection_sig: ConnectionSignature,
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, TypedBuilder)]
pub struct ResponseDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

/// Non-standalone message type.
/// This is only encountered as part of an existent message.
/// It is not a message on it's own.
#[derive(Copy, Clone, Debug, Deserialize, Default, PartialEq)]
#[serde(try_from = "CowStr")]
struct SigEd25519Sha512Single;

impl<'a> From<&'a SigEd25519Sha512Single> for SignatureTypeV1_0 {
    fn from(_value: &'a SigEd25519Sha512Single) -> Self {
        SignatureTypeV1_0::Ed25519Sha512Single
    }
}

impl<'a> TryFrom<CowStr<'a>> for SigEd25519Sha512Single {
    type Error = String;

    fn try_from(value: CowStr<'a>) -> Result<Self, Self::Error> {
        let value = MessageType::try_from(value.0.as_ref())?;

        if let Protocol::SignatureType(SignatureType::V1(SignatureTypeV1::V1_0(kind))) =
            value.protocol
        {
            if let Ok(SignatureTypeV1_0::Ed25519Sha512Single) = kind.kind_from_str(value.kind) {
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
        let protocol = Protocol::from(SignatureTypeV1_0::parent());
        let kind = SignatureTypeV1_0::from(self);
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
        msg_types::connection::ConnectionTypeV1_0,
    };

    #[test]
    fn test_minimal_conn_response() {
        let conn_sig = ConnectionSignature::new(
            "test_signature".to_owned(),
            "test_sig_data".to_owned(),
            "test_signer".to_owned(),
        );

        let content = ResponseContent::builder().connection_sig(conn_sig).build();

        let decorators = ResponseDecorators::builder()
            .thread(make_extended_thread())
            .build();

        let expected = json!({
            "connection~sig": content.connection_sig,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, ConnectionTypeV1_0::Response, expected);
    }

    #[test]
    fn test_extended_conn_response() {
        let conn_sig = ConnectionSignature::new(
            "test_signature".to_owned(),
            "test_sig_data".to_owned(),
            "test_signer".to_owned(),
        );

        let content = ResponseContent::builder().connection_sig(conn_sig).build();

        let decorators = ResponseDecorators::builder()
            .thread(make_extended_thread())
            .timing(make_extended_timing())
            .please_ack(make_minimal_please_ack())
            .build();

        let expected = json!({
            "connection~sig": content.connection_sig,
            "~thread": decorators.thread,
            "~timing": decorators.timing,
            "~please_ack": decorators.please_ack
        });

        test_utils::test_msg(content, decorators, ConnectionTypeV1_0::Response, expected);
    }
}
