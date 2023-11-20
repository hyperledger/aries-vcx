use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{thread::Thread, transport::Transport},
    msg_parts::MsgParts,
};

pub type StatusRequest = MsgParts<StatusRequestContent, StatusRequestDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct StatusRequestContent {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient_key: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct StatusRequestDecorators {
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
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{misc::test_utils, msg_types::protocols::pickup::PickupTypeV2_0};

    #[test]
    fn test_status_request() {
        let expected = json!(
            {
                "@id": "123456781",
                "@type": "https://didcomm.org/messagepickup/2.0/status-request",
                "recipient_key": "<key for messages>"
            }
        );
        let content = StatusRequestContent::builder()
            .recipient_key("<key for messages>".to_owned())
            .build();
        let decorators = StatusRequestDecorators::builder().build();

        test_utils::test_msg(content, decorators, PickupTypeV2_0::StatusRequest, expected);
    }
}
