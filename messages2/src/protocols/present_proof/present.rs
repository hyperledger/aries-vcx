use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Attachment, PleaseAck, Thread, Timing},
    msg_types::types::present_proof::PresentProofV1_0Kind,
    Message,
};

pub type Presentation = Message<PresentationContent, PresentationDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "PresentProofV1_0Kind::Presentation")]
pub struct PresentationContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "presentations~attach")]
    pub presentations_attach: Vec<Attachment>,
}

impl PresentationContent {
    pub fn new(presentations_attach: Vec<Attachment>) -> Self {
        Self {
            comment: None,
            presentations_attach,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PresentationDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

impl PresentationDecorators {
    pub fn new(thread: Thread) -> Self {
        Self {
            thread,
            please_ack: None,
            timing: None,
        }
    }
}
