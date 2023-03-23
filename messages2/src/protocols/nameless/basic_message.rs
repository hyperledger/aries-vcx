//! Module containing the `basic message` protocol messages, as defined in the [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/features/0095-basic-message/README.md).

use chrono::{DateTime, Utc};
use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{localization::MsgLocalization, thread::Thread, timing::Timing},
    misc::utils,
    msg_parts::MsgParts,
    msg_types::types::basic_message::BasicMessageV1_0,
};

pub type BasicMessage = MsgParts<BasicMessageContent, BasicMessageDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "BasicMessageV1_0::Message")]
pub struct BasicMessageContent {
    pub content: String,
    #[serde(serialize_with = "utils::serialize_datetime")]
    pub sent_time: DateTime<Utc>,
}

impl BasicMessageContent {
    pub fn new(content: String) -> Self {
        Self {
            content,
            sent_time: DateTime::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct BasicMessageDecorators {
    #[serde(rename = "~l10n")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l10n: Option<MsgLocalization>,
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::thread::tests::make_extended_thread,
        misc::test_utils::{self, DateTimeRfc3339},
    };

    #[test]
    fn test_minimal_basic_message() {
        let mut content = BasicMessageContent::new("test_content".to_owned());
        content.sent_time = DateTime::default();

        let decorators = BasicMessageDecorators::default();

        let expected = json!({
            "sent_time": DateTimeRfc3339(&content.sent_time),
            "content": content.content
        });

        test_utils::test_msg::<BasicMessageContent, _, _>(content, decorators, expected);
    }

    #[test]
    fn test_extended_basic_message() {
        let mut content = BasicMessageContent::new("test_content".to_owned());
        content.sent_time = DateTime::default();

        let mut decorators = BasicMessageDecorators::default();
        decorators.thread = Some(make_extended_thread());

        let expected = json!({
            "sent_time": DateTimeRfc3339(&content.sent_time),
            "content": content.content,
            "~thread": decorators.thread
        });

        test_utils::test_msg::<BasicMessageContent, _, _>(content, decorators, expected);
    }
}
