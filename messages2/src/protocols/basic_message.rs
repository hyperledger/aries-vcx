use chrono::{DateTime, Utc};
use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{MsgLocalization, Thread, Timing},
    misc::utils,
    msg_types::types::basic_message::BasicMessageV1_0Kind,
    Message,
};

pub type BasicMessage = Message<BasicMessageContent, BasicMessageDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "BasicMessageV1_0Kind::Message")]
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
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::{test_utils, utils::DATETIME_FORMAT};

    #[test]
    fn test_minimal_message() {
        let msg_str = "test".to_owned();

        let msg_type = test_utils::build_msg_type::<BasicMessageContent>();

        let mut content = BasicMessageContent::new(msg_str.clone());
        let datetime = DateTime::default();
        content.sent_time = datetime;

        let decorators = BasicMessageDecorators::default();

        let json = json!({
            "@type": msg_type,
            "sent_time": datetime.format(DATETIME_FORMAT).to_string(),
            "content": msg_str
        });

        test_utils::test_msg(content, decorators, json);
    }

    #[test]
    fn test_extensive_message() {
        let msg_str = "test".to_owned();

        let msg_type = test_utils::build_msg_type::<BasicMessageContent>();

        let mut content = BasicMessageContent::new(msg_str.clone());
        let datetime = DateTime::default();
        content.sent_time = datetime;

        let mut decorators = BasicMessageDecorators::default();
        let thid = "test".to_owned();
        let thread = Thread::new(thid.clone());
        decorators.thread = Some(thread);

        let json = json!({
            "@type": msg_type,
            "sent_time": datetime.format(DATETIME_FORMAT).to_string(),
            "content": msg_str,
            "~thread": {
                "thid": thid
            }
        });

        test_utils::test_msg(content, decorators, json);
    }
}
