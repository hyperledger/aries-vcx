use serde::{Deserialize, Serialize};

use crate::{
    decorators::{thread::Thread, timing::Timing},
    misc::NoDecorators,
    msg_parts::MsgParts,
};

pub type HandshakeReuse = MsgParts<HandshakeReuseContent, HandshakeReuseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(transparent)]
pub struct HandshakeReuseContent(NoDecorators);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct HandshakeReuseDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

impl HandshakeReuseDecorators {
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
        msg_types::out_of_band::OutOfBandTypeV1_1,
    };

    #[test]
    fn test_minimal_reuse() {
        let content = HandshakeReuseContent::default();

        let decorators = HandshakeReuseDecorators::new(make_extended_thread());

        let expected = json!({
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, OutOfBandTypeV1_1::HandshakeReuse, expected);
    }

    #[test]
    fn test_extended_reuse() {
        let content = HandshakeReuseContent::default();

        let mut decorators = HandshakeReuseDecorators::new(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, OutOfBandTypeV1_1::HandshakeReuse, expected);
    }
}
