use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_types::types::trust_ping::TrustPingV1_0Kind,
    Message,
};

pub type Ping = Message<PingContent, PingDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, Default, PartialEq)]
#[message(kind = "TrustPingV1_0Kind::Ping")]
pub struct PingContent {
    #[serde(default)]
    pub response_requested: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct PingDecorators {
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
    use crate::{decorators::thread::tests::make_extended_thread, misc::test_utils};

    #[test]
    fn test_minimal_ping() {
        let content = PingContent::default();

        let decorators = PingDecorators::default();

        let json = json!({
            "response_requested": false,
        });

        test_utils::test_msg::<PingContent, _, _>(content, decorators, json);
    }

    #[test]
    fn test_extensive_ping() {
        let mut content = PingContent::default();
        content.comment = Some("test_comment".to_owned());

        let mut decorators = PingDecorators::default();
        decorators.thread = Some(make_extended_thread());

        let json = json!({
            "response_requested": false,
            "comment": content.comment,
            "~thread": decorators.thread
        });

        test_utils::test_msg::<PingContent, _, _>(content, decorators, json);
    }
}
