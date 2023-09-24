use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type PingResponse = MsgParts<PingResponseContent, PingResponseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct PingResponseContent {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct PingResponseDecorators {
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
        msg_types::trust_ping::TrustPingTypeV1_0,
    };

    #[test]
    fn test_minimal_ping_response() {
        let content = PingResponseContent::default();

        let decorators = PingResponseDecorators::builder()
            .thread(make_extended_thread())
            .build();

        let expected = json!({
            "~thread": decorators.thread
        });

        test_utils::test_msg(
            content,
            decorators,
            TrustPingTypeV1_0::PingResponse,
            expected,
        );
    }

    #[test]
    fn test_extended_ping_response() {
        let content = PingResponseContent::builder()
            .comment("test_comment".to_owned())
            .build();

        let decorators = PingResponseDecorators::builder()
            .thread(make_extended_thread())
            .timing(make_extended_timing())
            .build();

        let expected = json!({
            "comment": content.comment,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(
            content,
            decorators,
            TrustPingTypeV1_0::PingResponse,
            expected,
        );
    }
}
