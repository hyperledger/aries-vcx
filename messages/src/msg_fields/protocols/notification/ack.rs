use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type Ack = MsgParts<AckContent, AckDecorators>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TypedBuilder)]
#[builder(build_method(into))]
pub struct AckContent {
    pub status: AckStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AckStatus {
    Ok,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TypedBuilder)]
pub struct AckDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[builder(default, setter(strip_option))]
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
        decorators::{thread::tests::make_extended_thread, timing::tests::make_extended_timing},
        misc::test_utils,
        msg_types::notification::NotificationTypeV1_0,
    };

    #[test]
    fn test_minimal_ack() {
        let content: AckContent = AckContent::builder().status(AckStatus::Ok).build();

        let decorators = AckDecorators::builder().thread(make_extended_thread()).build();

        let expected = json!({
            "status": content.status,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, NotificationTypeV1_0::Ack, expected);
    }

    #[test]
    fn test_extended_ack() {
        let content: AckContent = AckContent::builder().status(AckStatus::Ok).build();

        let decorators = AckDecorators::builder()
            .thread(make_extended_thread())
            .timing(make_extended_timing())
            .build();

        let expected = json!({
            "status": content.status,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, NotificationTypeV1_0::Ack, expected);
    }
}
