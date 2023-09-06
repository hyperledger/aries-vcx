use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type Ping = MsgParts<PingContent, PingDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct PingContent {
    #[serde(default)]
    pub response_requested: bool,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct PingDecorators {
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
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
        decorators::thread::tests::make_extended_thread, misc::test_utils, msg_types::trust_ping::TrustPingTypeV1_0,
    };

    #[test]
    fn test_minimal_ping() {
        let content = PingContent::default();

        let decorators = PingDecorators::default();

        let expected = json!({
            "response_requested": false,
        });

        test_utils::test_msg(content, decorators, TrustPingTypeV1_0::Ping, expected);
    }

    #[test]
    fn test_extended_ping() {
        let mut content = PingContent::default();
        content.comment = Some("test_comment".to_owned());

        let mut decorators = PingDecorators::default();
        decorators.thread = Some(make_extended_thread());

        let expected = json!({
            "response_requested": false,
            "comment": content.comment,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, TrustPingTypeV1_0::Ping, expected);
    }
}
