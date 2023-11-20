use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{attachment::Attachment, thread::Thread, transport::Transport},
    msg_parts::MsgParts,
};

pub type Delivery = MsgParts<DeliveryContent, DeliveryDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct DeliveryContent {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient_key: Option<String>,
    #[serde(rename = "~attach")]
    pub attach: Vec<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct DeliveryDecorators {
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
    use crate::{
        decorators::{
            attachment::{AttachmentData, AttachmentType},
            thread::Thread,
        },
        misc::test_utils,
        msg_types::protocols::pickup::PickupTypeV2_0,
    };
    #[test]
    fn test_delivery() {
        let expected = json!(
            {
                "@id": "123456781",
                "~thread": {
                    "thid": "<message id of delivery-request message>"
                  },
                "@type": "https://didcomm.org/messagepickup/2.0/delivery",
                "recipient_key": "<key for messages>",
                "~attach": [{
                    "@id": "<messageid>",
                    "data": {
                        "base64": ""
                    }
                }]
            }
        );
        let attach = Attachment::builder()
            .id("<messageid>".to_owned())
            .data(
                AttachmentData::builder()
                    .content(AttachmentType::Base64("".into()))
                    .build(),
            )
            .build();
        let content = DeliveryContent::builder()
            .recipient_key("<key for messages>".to_owned())
            .attach(vec![attach])
            .build();
        let decorators = DeliveryDecorators::builder()
            .thread(
                Thread::builder()
                    .thid("<message id of delivery-request message>".to_owned())
                    .build(),
            )
            .build();

        test_utils::test_msg(content, decorators, PickupTypeV2_0::Delivery, expected);
    }
}
