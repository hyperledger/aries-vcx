use messages_macros::Message;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Localization, Thread, Timing},
    message_type::message_family::basic_message::{BasicMessage as BasicMessageKind, BasicMessageV1, BasicMessageV1_0}, macros::threadlike_opt_impl,
};

use super::traits::ConcreteMessage;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "BasicMessageKind::V1(BasicMessageV1::V1_0(BasicMessageV1_0::Message))")]
pub struct BasicMessage {
    #[serde(rename = "@id")]
    pub id: String,
    pub sent_time: String,
    pub content: String,
    #[serde(rename = "~l10n")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l10n: Option<Localization>,
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

threadlike_opt_impl!(BasicMessage);