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
    #[serde(rename = "did_doc~attach", skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub did_doc: Option<Attachment>,
    #[serde(rename = "did_rotate~attach", skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub did_rotate: Option<Attachment>,
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
        misc::{test_utils, MimeType},
        msg_types::protocols::did_exchange::DidExchangeTypeV1_1,
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
            did_rotate: Some(
                Attachment::builder()
                    .data(
                        AttachmentData::builder()
                            .content(AttachmentType::Base64(String::from("Qi5kaWRAQjpB")))
                            .build(),
                    )
                    .mime_type(MimeType::Plain)
                    .build(),
            ),
        }
    }

    #[test]
    fn test_minimal_conn_response() {
        let mut content = response_content();
        content.did_doc = None;
        content.did_rotate = None;

        let decorators = ResponseDecorators {
            thread: make_extended_thread(),
            timing: None,
        };

        let expected = json!({
            "did": content.did,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, DidExchangeTypeV1_1::Response, expected);
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
            "did_rotate~attach": content.did_rotate,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, DidExchangeTypeV1_1::Response, expected);
    }
}
