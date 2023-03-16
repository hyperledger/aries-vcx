use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Thread, Timing},
    msg_types::types::notification::NotificationV1_0Kind,
    Message,
};

pub type Ack = Message<AckContent, AckDecorators>;

#[derive(Debug, Clone, Serialize, Deserialize, MessageContent, PartialEq)]
#[message(kind = "NotificationV1_0Kind::Ack")]
pub struct AckContent {
    pub status: AckStatus,
}

impl AckContent {
    pub fn new(status: AckStatus) -> Self {
        Self { status }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AckStatus {
    Ok,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AckDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

impl AckDecorators {
    pub fn new(thread: Thread) -> Self {
        Self { thread, timing: None }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{AriesMessage, Message};

    const ACK: &str = "https://didcomm.org/notification/1.0/ack";

    #[test]
    fn test_minimal_message() {
        let status = AckStatus::Ok;
        let thid = "test".to_owned();
        let id = "test".to_owned();

        let thread = Thread::new(thid.clone());

        let content = AckContent::new(status);

        let decorators = AckDecorators::new(thread);
        let msg = Message::with_decorators(id.clone(),content, decorators);
        let msg = AriesMessage::from(msg);

        let json = json!({
            "@type": ACK,
            "@id": id,
            "status": status,
            "~thread": {
                "thid": thid
            }
        });

        let deserialized = AriesMessage::deserialize(&json).unwrap();

        assert_eq!(serde_json::to_value(&msg).unwrap(), json);
        assert_eq!(deserialized, msg);
    }

    #[test]
    fn test_extensive_message() {
        let status = AckStatus::Ok;
        let thid = "test".to_owned();
        let in_time = "test".to_owned();
        let id = "test".to_owned();

        let thread = Thread::new(thid.clone());
        let mut timing = Timing::default();
        timing.in_time = Some(in_time.clone());

        let content = AckContent::new(status);

        let mut decorators = AckDecorators::new(thread);
        decorators.timing = Some(timing);

        let msg = Message::with_decorators(id.clone(), content, decorators);
        let msg = AriesMessage::from(msg);

        let json = json!({
            "@type": ACK,
            "@id": id,
            "status": status,
            "~thread": {
                "thid": thid
            },
            "~timing": {
                "in_time": in_time
            }
        });

        let deserialized = AriesMessage::deserialize(&json).unwrap();

        assert_eq!(serde_json::to_value(&msg).unwrap(), json);
        assert_eq!(deserialized, msg);
    }

    #[test]
    #[should_panic]
    fn test_incomplete_message() {
        let status = AckStatus::Ok;

        let json = json!({
            "@type": ACK,
            "@id": "test",
            "status": status,
        });

        AriesMessage::deserialize(&json).unwrap();
    }
}
