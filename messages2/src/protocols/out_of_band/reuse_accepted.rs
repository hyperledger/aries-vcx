use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::{Message, Nothing},
    decorators::{Thread, Timing},
    message_type::message_family::out_of_band::OutOfBandV1_1,
    protocols::traits::MessageKind,
};

pub type HandshakeReuseAccepted = Message<HandshakeReuseAcceptedContent, HandshakeReuseAcceptedDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "OutOfBandV1_1::HandshakeReuseAccepted")]
#[serde(transparent)]
pub struct HandshakeReuseAcceptedContent(Nothing);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HandshakeReuseAcceptedDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
