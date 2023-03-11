use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::{Message, Nothing},
    decorators::{Thread, Timing},
    message_type::message_protocol::out_of_band::OutOfBandV1_1Kind,
    protocols::traits::ConcreteMessage,
};

pub type HandshakeReuse = Message<HandshakeReuseContent, HandshakeReuseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, Default)]
#[message(kind = "OutOfBandV1_1Kind::HandshakeReuse")]
#[serde(transparent)]
pub struct HandshakeReuseContent(Nothing);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HandshakeReuseDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

impl HandshakeReuseDecorators {
    pub fn new(thread: Thread) -> Self {
        Self { thread, timing: None }
    }
}
