use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{thread::Thread, timing::Timing},
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
    use crate::{
        decorators::{thread::tests::make_extended_thread, timing::tests::make_extended_timing},
        misc::test_utils,
    };

    #[test]
    fn test_minimal_ping_response() {
        let msg_type = test_utils::build_msg_type::<PingResponseContent>();

        let content = PingResponseContent::default();

        let decorators = PingResponseDecorators::new(make_extended_thread());

        let json = json!({
            "@type": msg_type,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, json);
    }

    #[test]
    fn test_extensive_ping_response() {
        let msg_type = test_utils::build_msg_type::<PingResponseContent>();

        let mut content = PingResponseContent::default();
        content.comment = Some("test_comment".to_owned());

        let mut decorators = PingResponseDecorators::new(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let json = json!({
            "@type": msg_type,
            "comment": content.comment,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, json);
    }
}
