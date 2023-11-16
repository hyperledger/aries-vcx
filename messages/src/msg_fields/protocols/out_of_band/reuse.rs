use serde::{Deserialize, Serialize};
use shared::misc::serde_ignored::SerdeIgnored;
use typed_builder::TypedBuilder;

use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type HandshakeReuse = MsgParts<HandshakeReuseContent, HandshakeReuseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
#[serde(transparent)]
pub struct HandshakeReuseContent {
    #[builder(default, setter(skip))]
    inner: SerdeIgnored,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct HandshakeReuseDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[cfg(test)]
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
    fn test_minimal_reuse() {
        let content = HandshakeReuseContent::default();

        let decorators = HandshakeReuseDecorators::builder()
            .thread(make_extended_thread())
            .build();

        let expected = json!({
            "~thread": decorators.thread
        });

        test_utils::test_msg(
            content,
            decorators,
            OutOfBandTypeV1_1::HandshakeReuse,
            expected,
        );
    }

    #[test]
    fn test_extended_reuse() {
        let content = HandshakeReuseContent::default();

        let decorators = HandshakeReuseDecorators::builder()
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
            OutOfBandTypeV1_1::HandshakeReuse,
            expected,
        );
    }
}
