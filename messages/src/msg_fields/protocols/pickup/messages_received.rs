use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{thread::Thread, transport::Transport},
    msg_parts::MsgParts,
};

pub type MessagesReceived = MsgParts<MessagesReceivedContent, MessagesReceivedDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct MessagesReceivedContent {
    pub message_id_list: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct MessagesReceivedDecorators {
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
        let decorators = MessagesReceivedDecorators::builder().build();

        test_utils::test_msg(
            content,
            decorators,
            PickupTypeV2_0::MessagesReceived,
            expected,
        );
    }
}
