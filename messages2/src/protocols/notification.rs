use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Thread, Timing},
    msg_types::types::notification::NotificationV1_0Kind,
    Message,
};

pub type Ack = Message<AckContent, AckDecorators>;

#[derive(Debug, Clone, Serialize, Deserialize, MessageContent)]
#[message(kind = "NotificationV1_0Kind::Ack")]
pub struct AckContent {
    pub status: AckStatus,
}

impl AckContent {
    pub fn new(status: AckStatus) -> Self {
        Self { status }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AckStatus {
    Ok,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

impl AckDecorators {
    pub fn new(thread: Thread) -> Self {
        Self { thread, timing: None }
    }
}
