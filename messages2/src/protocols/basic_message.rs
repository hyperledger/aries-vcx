use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{MsgLocalization, Thread, Timing},
    message_type::message_family::basic_message::{BasicMessage as BasicMessageKind, BasicMessageV1, BasicMessageV1_0}, composite_message::Message,
};

use super::traits::MessageKind;

pub type BasicMessage = Message<BasicMessageContent, BasicMessageDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "BasicMessageKind::V1(BasicMessageV1::V1_0(BasicMessageV1_0::Message))")]
pub struct BasicMessageContent {
    pub sent_time: String,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BasicMessageDecorators {
    #[serde(rename = "~l10n")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l10n: Option<MsgLocalization>,
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
