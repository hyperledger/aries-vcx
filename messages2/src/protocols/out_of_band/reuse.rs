use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{thread::Thread, timing::Timing},
    misc::nothing::Nothing,
    msg_types::types::out_of_band::OutOfBandV1_1Kind,
    Message,
};

pub type HandshakeReuse = Message<HandshakeReuseContent, HandshakeReuseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, Default, PartialEq)]
#[message(kind = "OutOfBandV1_1Kind::HandshakeReuse")]
#[serde(transparent)]
pub struct HandshakeReuseContent(Nothing);

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
    };

    #[test]
    fn test_minimal_reuse() {
        let msg_type = test_utils::build_msg_type::<HandshakeReuseContent>();

        let content = HandshakeReuseContent::default();

        let decorators = HandshakeReuseDecorators::new(make_extended_thread());

        let json = json!({
            "@type": msg_type,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, json);
    }

    #[test]
    fn test_extensive_reuse() {
        let msg_type = test_utils::build_msg_type::<HandshakeReuseContent>();

        let content = HandshakeReuseContent::default();

        let mut decorators = HandshakeReuseDecorators::new(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let json = json!({
            "@type": msg_type,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, json);
    }
}
