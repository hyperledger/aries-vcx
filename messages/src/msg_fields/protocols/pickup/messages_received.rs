use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::decorators::PickupDecoratorsCommon;
use crate::msg_parts::MsgParts;

pub type MessagesReceived = MsgParts<MessagesReceivedContent, PickupDecoratorsCommon>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct MessagesReceivedContent {
    pub message_id_list: Vec<String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{misc::test_utils, msg_types::protocols::pickup::PickupTypeV2_0};
    #[test]
    fn test_messages_received() {
        let expected = json!(
            {
                "@type": "https://didcomm.org/messagepickup/2.0/messages-received",
                "message_id_list": ["123","456"]
            }

        );
        let content = MessagesReceivedContent::builder()
            .message_id_list(vec!["123".to_string(), "456".to_string()])
            .build();
        let decorators = PickupDecoratorsCommon::builder().build();

        test_utils::test_msg(
            content,
            decorators,
            PickupTypeV2_0::MessagesReceived,
            expected,
        );
    }
}
