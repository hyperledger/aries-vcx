use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{thread::Thread, transport::Transport},
    msg_parts::MsgParts,
};

pub type LiveDeliveryChange = MsgParts<LiveDeliveryChangeContent, LiveDeliveryChangeDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct LiveDeliveryChangeContent {
    pub live_delivery: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct LiveDeliveryChangeDecorators {
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~transport")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<Transport>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
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
        let decorators = LiveDeliveryChangeDecorators::builder().build();

        test_utils::test_msg(
            content,
            decorators,
            PickupTypeV2_0::LiveDeliveryChange,
            expected,
        );
    }
}
