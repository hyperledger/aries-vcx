use serde::{Deserialize, Serialize};

use crate::{
    decorators::{thread::Thread, timing::Timing},
    misc::NoDecorators,
    msg_parts::MsgParts,
};

pub type HandshakeReuseAccepted = MsgParts<HandshakeReuseAcceptedContent, HandshakeReuseAcceptedDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(transparent)]
pub struct HandshakeReuseAcceptedContent(NoDecorators);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct HandshakeReuseAcceptedDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

impl HandshakeReuseAcceptedDecorators {
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
        misc::test_utils, msg_types::out_of_band::OutOfBandProtocolV1_1,
    };

    #[test]
    fn test_minimal_reuse_accepted() {
        let content = HandshakeReuseAcceptedContent::default();

        let decorators = HandshakeReuseAcceptedDecorators::new(make_extended_thread());

        let expected = json!({
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, OutOfBandProtocolV1_1::HandshakeReuseAccepted, expected);
    }

    #[test]
    fn test_extended_reuse_accepted() {
        let content = HandshakeReuseAcceptedContent::default();

        let mut decorators = HandshakeReuseAcceptedDecorators::new(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, OutOfBandProtocolV1_1::HandshakeReuseAccepted, expected);
    }
}
