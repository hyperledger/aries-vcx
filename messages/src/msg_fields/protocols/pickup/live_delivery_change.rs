use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::decorators::PickupDecoratorsCommon;
use crate::msg_parts::MsgParts;

pub type LiveDeliveryChange = MsgParts<LiveDeliveryChangeContent, PickupDecoratorsCommon>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct LiveDeliveryChangeContent {
    pub live_delivery: bool,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{misc::test_utils, msg_types::protocols::pickup::PickupTypeV2_0};
    #[test]
    fn test_live_delivery_change() {
        let expected = json!(
            {
                "@type": "https://didcomm.org/messagepickup/2.0/live-delivery-change",
                "live_delivery": true
            }
        );
        let content = LiveDeliveryChangeContent::builder()
            .live_delivery(true)
            .build();
        let decorators = PickupDecoratorsCommon::builder().build();

        test_utils::test_msg(
            content,
            decorators,
            PickupTypeV2_0::LiveDeliveryChange,
            expected,
        );
    }
}
