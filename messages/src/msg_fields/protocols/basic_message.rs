//! Module containing the `basic message` protocol messages, as defined in the [RFC](<https://github.com/hyperledger/aries-rfcs/blob/main/features/0095-basic-message/README.md>).

use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{localization::MsgLocalization, thread::Thread, timing::Timing},
    misc::utils::{self, into_msg_with_type},
    msg_parts::MsgParts,
    msg_types::protocols::basic_message::BasicMessageTypeV1_0,
};

pub type BasicMessage = MsgParts<BasicMessageContent, BasicMessageDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct BasicMessageContent {
    pub content: String,
    #[serde(serialize_with = "utils::serialize_datetime")]
    pub sent_time: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct BasicMessageDecorators {
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~l10n")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l10n: Option<MsgLocalization>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

into_msg_with_type!(BasicMessage, BasicMessageTypeV1_0, Message);

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
        let content = BasicMessageContent::builder()
            .content("test_content".to_owned())
            .sent_time(DateTime::default())
            .build();

        let decorators = BasicMessageDecorators::default();

        let expected = json!({
            "sent_time": DateTimeRfc3339(&content.sent_time),
            "content": content.content
        });

        test_utils::test_msg(content, decorators, BasicMessageTypeV1_0::Message, expected);
    }

    #[test]
    fn test_extended_basic_message() {
        let content = BasicMessageContent::builder()
            .content("test_content".to_owned())
            .sent_time(DateTime::default())
            .build();

        let mut decorators = BasicMessageDecorators::default();
        decorators.thread = Some(make_extended_thread());

        let expected = json!({
            "sent_time": DateTimeRfc3339(&content.sent_time),
            "content": content.content,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, BasicMessageTypeV1_0::Message, expected);
    }
}
