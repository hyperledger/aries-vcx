use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{thread::Thread, timing::Timing},
    message::Message,
    msg_types::types::notification::NotificationV1_0,
};

pub type Ack = Message<AckContent, AckDecorators>;

#[derive(Debug, Clone, Serialize, Deserialize, MessageContent, PartialEq)]
#[message(kind = "NotificationV1_0::Ack")]
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
    use crate::{
        decorators::{thread::tests::make_extended_thread, timing::tests::make_extended_timing},
        misc::test_utils,
    };

    #[test]
    fn test_minimal_ack() {
        let content = AckContent::new(AckStatus::Ok);

        let decorators = AckDecorators::new(make_extended_thread());

        let json = json!({
            "status": content.status,
            "~thread": decorators.thread
        });

        test_utils::test_msg::<AckContent, _, _>(content, decorators, json);
    }

    #[test]
    fn test_extensive_ack() {
        let content = AckContent::new(AckStatus::Ok);

        let mut decorators = AckDecorators::new(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let json = json!({
            "status": content.status,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg::<AckContent, _, _>(content, decorators, json);
    }
}
