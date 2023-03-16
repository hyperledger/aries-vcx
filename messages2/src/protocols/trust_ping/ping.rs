use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Thread, Timing},
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
    use crate::misc::test_utils;

    #[test]
    fn test_minimal_message() {
        let msg_type = test_utils::build_msg_type::<PingContent>();

        let content = PingContent::default();

        let decorators = PingDecorators::default();

        let json = json!({
            "@type": msg_type,
            "response_requested": false,
        });

        test_utils::test_msg(content, decorators, json);
    }

    #[test]
    fn test_extensive_message() {
        let msg_type = test_utils::build_msg_type::<PingContent>();

        let mut content = PingContent::default();
        let comment_str = "test".to_owned();
        content.comment = Some(comment_str.clone());

        let mut decorators = PingDecorators::default();
        let thid = "test".to_owned();
        let thread = Thread::new(thid.clone());
        decorators.thread = Some(thread);

        let json = json!({
            "@type": msg_type,
            "response_requested": false,
            "comment": comment_str,
            "~thread": {
                "thid": thid
            }
        });

        test_utils::test_msg(content, decorators, json);
    }
}
