use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::Message,
    decorators::{Attachment, Thread, Timing},
    message_type::message_family::present_proof::PresentProofV1_0,
    protocols::traits::MessageKind,
};

pub type RequestPresentation = Message<RequestPresentationContent, RequestPresentationDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "PresentProofV1_0::RequestPresentation")]
pub struct RequestPresentationContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "request_presentations~attach")]
    pub request_presentations_attach: Vec<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestPresentationDecorators {
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
