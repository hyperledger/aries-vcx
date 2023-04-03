use serde::{Deserialize, Serialize};

use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type PingResponse = MsgParts<PingResponseContent, PingResponseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
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
        msg_types::trust_ping::TrustPingTypeV1_0,
    };

    #[test]
    fn test_minimal_ping_response() {
        let content = PingResponseContent::default();

        let decorators = PingResponseDecorators::new(make_extended_thread());

        let expected = json!({
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, TrustPingTypeV1_0::PingResponse, expected);
    }

    #[test]
    fn test_extended_ping_response() {
        let mut content = PingResponseContent::default();
        content.comment = Some("test_comment".to_owned());

        let mut decorators = PingResponseDecorators::new(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "comment": content.comment,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, TrustPingTypeV1_0::PingResponse, expected);
    }
}
