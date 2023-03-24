use serde::{Deserialize, Serialize};

use crate::{
    decorators::{attachment::Attachment, thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type RequestPresentation = MsgParts<RequestPresentationContent, RequestPresentationDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RequestPresentationContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "request_presentations~attach")]
    pub request_presentations_attach: Vec<Attachment>,
}

impl RequestPresentationContent {
    pub fn new(request_presentations_attach: Vec<Attachment>) -> Self {
        Self {
            comment: None,
            request_presentations_attach,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct RequestPresentationDecorators {
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
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
        decorators::{
            attachment::tests::make_extended_attachment, thread::tests::make_extended_thread,
            timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_types::present_proof::PresentProofProtocolV1_0,
    };

    #[test]
    fn test_minimal_request_proof() {
        let content = RequestPresentationContent::new(vec![make_extended_attachment()]);

        let decorators = RequestPresentationDecorators::default();

        let expected = json!({
            "request_presentations~attach": content.request_presentations_attach,
        });

        test_utils::test_msg(
            content,
            decorators,
            PresentProofProtocolV1_0::RequestPresentation,
            expected,
        );
    }

    #[test]
    fn test_extended_request_proof() {
        let mut content = RequestPresentationContent::new(vec![make_extended_attachment()]);
        content.comment = Some("test_comment".to_owned());

        let mut decorators = RequestPresentationDecorators::default();
        decorators.thread = Some(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "request_presentations~attach": content.request_presentations_attach,
            "comment": content.comment,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(
            content,
            decorators,
            PresentProofProtocolV1_0::RequestPresentation,
            expected,
        );
    }
}
