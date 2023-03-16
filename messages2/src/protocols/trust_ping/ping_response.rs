use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Thread, Timing},
    msg_types::types::trust_ping::TrustPingV1_0Kind,
    Message,
};

pub type PingResponse = Message<PingResponseContent, PingResponseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, Default, PartialEq)]
#[message(kind = "TrustPingV1_0Kind::PingResponse")]
pub struct PingResponseContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PingResponseDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

impl PingResponseDecorators {
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
        let msg_type = test_utils::build_msg_type::<PingResponseContent>();

        let content = PingResponseContent::default();

        let thid = "test".to_owned();
        let thread = Thread::new(thid.clone());
        let decorators = PingResponseDecorators::new(thread);

        let json = json!({
            "@type": msg_type,
            "~thread": {
                "thid": thid
            }
        });

        test_utils::test_msg(content, decorators, json);
    }

    #[test]
    fn test_extensive_message() {
        let msg_type = test_utils::build_msg_type::<PingResponseContent>();

        let mut content = PingResponseContent::default();
        let comment_str = "test".to_owned();
        content.comment = Some(comment_str.clone());

        let thid = "test".to_owned();
        let thread = Thread::new(thid.clone());
        let mut decorators = PingResponseDecorators::new(thread);
        let in_time = "test".to_owned();
        let mut timing = Timing::default();
        timing.in_time = Some(in_time.clone());
        decorators.timing = Some(timing);

        let json = json!({
            "@type": msg_type,
            "comment": comment_str,
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
