use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::decorators::PickupDecoratorsCommon;
use crate::msg_parts::MsgParts;

pub type StatusRequest = MsgParts<StatusRequestContent, PickupDecoratorsCommon>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct StatusRequestContent {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient_key: Option<String>,
}
