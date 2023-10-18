use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::decorators::PickupDecoratorsCommon;
use crate::msg_parts::MsgParts;

pub type LiveDeliveryChange = MsgParts<LiveDeliveryChangeContent, PickupDecoratorsCommon>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct LiveDeliveryChangeContent {
    pub live_delivery: bool,
}
