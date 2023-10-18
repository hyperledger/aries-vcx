use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::decorators::PickupDecoratorsCommon;
use crate::msg_parts::MsgParts;

pub type MessagesReceived = MsgParts<MessagesReceivedContent, PickupDecoratorsCommon>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct MessagesReceivedContent {
    pub message_id_list: Vec<String>,
}
