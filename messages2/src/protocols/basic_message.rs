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
    use crate::{misc::utils::DATETIME_FORMAT, AriesMessage, Message};

    const MESSAGE: &str = "https://didcomm.org/basicmessage/1.0/message";

    #[test]
    fn test_minimal_message() {
        let datetime = DateTime::default();
        let msg_str = "test".to_owned();
        let id = "test".to_owned();

        let mut content = BasicMessageContent::new(msg_str.clone());
        content.sent_time = datetime;

        let decorators = BasicMessageDecorators::default();
        let msg = Message::with_decorators(id.clone(), content, decorators);
        let msg = AriesMessage::from(msg);

        let json = json!({
            "@type": MESSAGE,
            "@id": id,
            "sent_time": datetime.format(DATETIME_FORMAT).to_string(),
            "content": msg_str
        });

        let deserialized = AriesMessage::deserialize(&json).unwrap();

        assert_eq!(serde_json::to_value(&msg).unwrap(), json);
        assert_eq!(deserialized, msg);
    }

    #[test]
    fn test_extensive_message() {
        let datetime = DateTime::default();
        let msg_str = "test".to_owned();
        let thid = "test".to_owned();
        let id = "test".to_owned();

        let mut content = BasicMessageContent::new(msg_str.clone());
        content.sent_time = datetime;

        let mut decorators = BasicMessageDecorators::default();
        let thread = Thread::new(thid.clone());
        decorators.thread = Some(thread);

        let msg = Message::with_decorators(id.clone(), content, decorators);
        let msg = AriesMessage::from(msg);

        let json = json!({
            "@type": MESSAGE,
            "@id": id,
            "sent_time": datetime.format(DATETIME_FORMAT).to_string(),
            "content": msg_str,
            "~thread": {
                "thid": thid
            }
        });

        let deserialized = AriesMessage::deserialize(&json).unwrap();

        assert_eq!(serde_json::to_value(&msg).unwrap(), json);
        assert_eq!(deserialized, msg);
    }

    #[test]
    #[should_panic]
    fn test_incomplete_message() {
        let datetime = DateTime::<Utc>::default();

        let json = json!({
            "@type": MESSAGE,
            "@id": "test",
            "sent_time": datetime.format(DATETIME_FORMAT).to_string()
        });

        AriesMessage::deserialize(&json).unwrap();
    }
}
