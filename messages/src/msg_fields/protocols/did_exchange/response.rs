use serde::{Deserialize, Serialize};

use crate::{
    decorators::{attachment::Attachment, thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type Response = MsgParts<ResponseContent, ResponseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ResponseContent {
    pub did: String, // TODO: Use Did
    #[serde(rename = "did_doc~attach")]
    pub did_doc: Option<Attachment>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ResponseDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use diddoc_legacy::aries::diddoc::AriesDidDoc;
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{
            attachment::{AttachmentData, AttachmentType},
            thread::tests::make_extended_thread,
            timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_types::protocols::did_exchange::DidExchangeTypeV1_0,
    };

    fn response_content() -> ResponseContent {
        let did_doc = AriesDidDoc::default();
        ResponseContent {
            did: did_doc.id.clone(),
            did_doc: Some(Attachment::new(AttachmentData::new(AttachmentType::Json(
                serde_json::to_value(did_doc).unwrap(),
            )))),
        }
    }

    #[test]
    fn test_minimal_conn_response() {
        let content = response_content();

        let decorators = ResponseDecorators {
            thread: make_extended_thread(),
            timing: None,
        };

        let expected = json!({
            "did": content.did,
            "did_doc~attach": content.did_doc,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, DidExchangeTypeV1_0::Response, expected);
    }

    #[test]
    fn test_extended_conn_response() {
        let content = response_content();

        let decorators = ResponseDecorators {
            thread: make_extended_thread(),
            timing: Some(make_extended_timing()),
        };

        let expected = json!({
            "did": content.did,
            "did_doc~attach": content.did_doc,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, DidExchangeTypeV1_0::Response, expected);
    }
}
