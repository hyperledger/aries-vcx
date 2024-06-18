use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::attachment::Attachment,
    msg_fields::protocols::did_exchange::v1_x::response::ResponseDecorators, msg_parts::MsgParts,
};

pub type Response = MsgParts<ResponseContent, ResponseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct ResponseContent {
    pub did: String, // TODO: Use Did
    #[serde(rename = "did_doc~attach")]
    pub did_doc: Option<Attachment>,
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
            did_doc: Some(
                Attachment::builder()
                    .data(
                        AttachmentData::builder()
                            .content(AttachmentType::Json(
                                serde_json::to_value(&did_doc).unwrap(),
                            ))
                            .build(),
                    )
                    .build(),
            ),
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
