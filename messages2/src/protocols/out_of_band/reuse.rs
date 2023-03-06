use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::{Nothing, Message},
    decorators::{Thread, Timing},
    message_type::message_family::out_of_band::OutOfBandV1_1,
    protocols::traits::MessageKind,
};

pub type HandshakeReuse = Message<HandshakeReuseContent, HandshakeReuseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "OutOfBandV1_1::HandshakeReuse")]
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
