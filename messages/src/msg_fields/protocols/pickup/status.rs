use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::msg_parts::MsgParts;

pub type Status = MsgParts<StatusContent>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct StatusContent {
    pub message_count: u32,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient_key: Option<String>,
}
