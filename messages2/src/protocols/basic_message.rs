use chrono::{DateTime, Utc};
use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{MsgLocalization, Thread, Timing},
    msg_types::types::basic_message::BasicMessageV1_0Kind,
    Message,
};

pub type BasicMessage = Message<BasicMessageContent, BasicMessageDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "BasicMessageV1_0Kind::Message")]
pub struct BasicMessageContent {
    pub content: String,
    pub sent_time: DateTime<Utc>,
}

impl BasicMessageContent {
    pub fn new(content: String) -> Self {
        Self {
            content,
            sent_time: DateTime::default(),
        }
    }
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
