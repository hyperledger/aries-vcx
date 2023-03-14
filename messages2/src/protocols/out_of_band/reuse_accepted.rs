use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::{Message},
    decorators::{Thread, Timing},
    msg_types::types::out_of_band::OutOfBandV1_1Kind,
    protocols::traits::ConcreteMessage, misc::nothing::Nothing,
};

pub type HandshakeReuseAccepted = Message<HandshakeReuseAcceptedContent, HandshakeReuseAcceptedDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, Default)]
#[message(kind = "OutOfBandV1_1Kind::HandshakeReuseAccepted")]
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

impl HandshakeReuseAcceptedDecorators {
    pub fn new(thread: Thread) -> Self {
        Self { thread, timing: None }
    }
}
