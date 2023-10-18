use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::decorators::PickupDecoratorsCommon;
use crate::msg_parts::MsgParts;

pub type Status = MsgParts<StatusContent, PickupDecoratorsCommon>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct StatusContent {
    pub message_count: u32,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient_key: Option<String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{misc::test_utils, msg_types::protocols::pickup::PickupTypeV2_0};

    #[test]
    fn test_status() {
        let expected = json!(
            {
                "@id": "123456781",
                "@type": "https://didcomm.org/messagepickup/2.0/status",
                "recipient_key": "<key for messages>",
                "message_count": 7,
            }
        );
        let content = StatusContent::builder()
            .recipient_key("<key for messages>".to_owned())
            .message_count(7)
            .build();
        let decorators = PickupDecoratorsCommon::builder().build();

        test_utils::test_msg(content, decorators, PickupTypeV2_0::Status, expected);
    }
}
