use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{attachment::Attachment, please_ack::PleaseAck, thread::Thread, timing::Timing},
    msg_types::types::present_proof::PresentProofV1_0Kind,
    Message,
};

pub type Presentation = Message<PresentationContent, PresentationDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "PresentProofV1_0Kind::Presentation")]
pub struct PresentationContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "presentations~attach")]
    pub presentations_attach: Vec<Attachment>,
}

impl PresentationContent {
    pub fn new(presentations_attach: Vec<Attachment>) -> Self {
        Self {
            comment: None,
            presentations_attach,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PresentationDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

impl PresentationDecorators {
    pub fn new(thread: Thread) -> Self {
        Self {
            thread,
            please_ack: None,
            timing: None,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{
            attachment::tests::make_extended_attachment, please_ack::tests::make_minimal_please_ack,
            thread::tests::make_extended_thread, timing::tests::make_extended_timing,
        },
        misc::test_utils,
    };

    #[test]
    fn test_minimal_present_proof() {
        let content = PresentationContent::new(vec![make_extended_attachment()]);

        let decorators = PresentationDecorators::new(make_extended_thread());

        let json = json!({
            "presentations~attach": content.presentations_attach,
            "~thread": decorators.thread
        });

        test_utils::test_msg::<PresentationContent, _, _,>(content, decorators, json);
    }

    #[test]
    fn test_extensive_present_proof() {
        let mut content = PresentationContent::new(vec![make_extended_attachment()]);
        content.comment = Some("test_comment".to_owned());

        let mut decorators = PresentationDecorators::new(make_extended_thread());
        decorators.timing = Some(make_extended_timing());
        decorators.please_ack = Some(make_minimal_please_ack());

        let json = json!({
            "comment": content.comment,
            "presentations~attach": content.presentations_attach,
            "~thread": decorators.thread,
            "~timing": decorators.timing,
            "~please_ack": decorators.please_ack
        });

        test_utils::test_msg::<PresentationContent, _, _,>(content, decorators, json);
    }
}
