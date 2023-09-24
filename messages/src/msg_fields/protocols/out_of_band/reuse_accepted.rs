use serde::{Deserialize, Serialize};
use shared_vcx::misc::serde_ignored::SerdeIgnored;
use typed_builder::TypedBuilder;

use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type HandshakeReuseAccepted =
    MsgParts<HandshakeReuseAcceptedContent, HandshakeReuseAcceptedDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
#[serde(transparent)]
pub struct HandshakeReuseAcceptedContent {
    #[builder(default, setter(skip))]
    inner: SerdeIgnored,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct HandshakeReuseAcceptedDecorators {
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
        msg_types::out_of_band::OutOfBandTypeV1_1,
    };

    #[test]
    fn test_minimal_reuse_accepted() {
        let content = HandshakeReuseAcceptedContent::default();

        let decorators = HandshakeReuseAcceptedDecorators::builder()
            .thread(make_extended_thread())
            .build();

        let expected = json!({
            "~thread": decorators.thread
        });

        test_utils::test_msg(
            content,
            decorators,
            OutOfBandTypeV1_1::HandshakeReuseAccepted,
            expected,
        );
    }

    #[test]
    fn test_extended_reuse_accepted() {
        let content = HandshakeReuseAcceptedContent::default();

        let decorators = HandshakeReuseAcceptedDecorators::builder()
            .thread(make_extended_thread())
            .timing(make_extended_timing())
            .build();

        let expected = json!({
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(
            content,
            decorators,
            OutOfBandTypeV1_1::HandshakeReuseAccepted,
            expected,
        );
    }
}
