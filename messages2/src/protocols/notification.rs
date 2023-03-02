use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Thread, Timing},
    message_type::message_family::notification::{Notification, NotificationV1, NotificationV1_0},
};

use super::traits::MessageKind;

#[derive(Debug, Clone, Serialize, Deserialize, MessageContent)]
#[message(kind = "Notification::V1(NotificationV1::V1_0(NotificationV1_0::Ack))")]
pub struct Ack {
    pub status: AckStatus,
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
