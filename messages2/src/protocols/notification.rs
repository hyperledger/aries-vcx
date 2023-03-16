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
    use crate::misc::test_utils;

    #[test]
    fn test_minimal_message() {
        let msg_type = test_utils::build_msg_type::<AckContent>();

        let status = AckStatus::Ok;
        let content = AckContent::new(status);

        let thid = "test".to_owned();
        let thread = Thread::new(thid.clone());
        let decorators = AckDecorators::new(thread);

        let json = json!({
            "@type": msg_type,
            "status": status,
            "~thread": {
                "thid": thid
            }
        });

        test_utils::test_msg(content, decorators, json);
    }

    #[test]
    fn test_extensive_message() {
        let msg_type = test_utils::build_msg_type::<AckContent>();

        let status = AckStatus::Ok;
        let content = AckContent::new(status);

        let thid = "test".to_owned();
        let thread = Thread::new(thid.clone());
        let mut decorators = AckDecorators::new(thread);
        let in_time = "test".to_owned();
        let mut timing = Timing::default();
        timing.in_time = Some(in_time.clone());
        decorators.timing = Some(timing);

        let json = json!({
            "@type": msg_type,
            "status": status,
            "~thread": {
                "thid": thid
            },
            "~timing": {
                "in_time": in_time
            }
        });

        test_utils::test_msg(content, decorators, json);
    }
}
