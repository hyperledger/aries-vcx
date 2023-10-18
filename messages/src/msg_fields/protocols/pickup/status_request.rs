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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
        let decorators = PickupDecoratorsCommon::builder().build();

        test_utils::test_msg(content, decorators, PickupTypeV2_0::StatusRequest, expected);
    }
}
