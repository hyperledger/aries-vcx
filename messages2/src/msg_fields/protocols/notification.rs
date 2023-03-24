//! Module containing the `acks` messages, as defined in the [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/features/0015-acks/README.md).

use serde::{Deserialize, Serialize};

use crate::{
    decorators::{thread::Thread, timing::Timing},
    misc::utils::into_msg_with_type,
    msg_parts::MsgParts,
    msg_types::protocols::notification::NotificationTypeV1_0,
};

pub type Ack = MsgParts<AckContent, AckDecorators>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

into_msg_with_type!(Ack, NotificationTypeV1_0, Ack);

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

        let expected = json!({
            "status": content.status,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, NotificationTypeV1_0::Ack, expected);
    }

    #[test]
    fn test_extended_ack() {
        let content = AckContent::new(AckStatus::Ok);

        let mut decorators = AckDecorators::new(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "status": content.status,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, NotificationTypeV1_0::Ack, expected);
    }
}
