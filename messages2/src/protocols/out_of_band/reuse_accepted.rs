use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{thread::Thread, timing::Timing},
    message::Message,
    misc::nothing::Nothing,
    msg_types::types::out_of_band::OutOfBandV1_1Kind,
};

pub type HandshakeReuseAccepted = Message<HandshakeReuseAcceptedContent, HandshakeReuseAcceptedDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, Default, PartialEq)]
#[message(kind = "OutOfBandV1_1Kind::HandshakeReuseAccepted")]
#[serde(transparent)]
pub struct HandshakeReuseAcceptedContent(Nothing);

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
        misc::test_utils,
    };

    #[test]
    fn test_minimal_reuse_accepted() {
        let content = HandshakeReuseAcceptedContent::default();

        let decorators = HandshakeReuseAcceptedDecorators::new(make_extended_thread());

        let json = json!({
            "~thread": decorators.thread
        });

        test_utils::test_msg::<HandshakeReuseAcceptedContent, _, _>(content, decorators, json);
    }

    #[test]
    fn test_extensive_reuse_accepted() {
        let content = HandshakeReuseAcceptedContent::default();

        let mut decorators = HandshakeReuseAcceptedDecorators::new(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let json = json!({
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg::<HandshakeReuseAcceptedContent, _, _>(content, decorators, json);
    }
}
