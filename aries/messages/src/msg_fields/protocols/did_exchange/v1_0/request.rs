use crate::{
    msg_fields::protocols::did_exchange::v1_x::request::{RequestContent, RequestDecorators},
    msg_parts::MsgParts,
};

/// Alias type for DIDExchange v1.0 Request message.
/// Note that since this inherits from the V1.X message, the direct serialization
/// of this Request is not recommended, as it will be indistinguisable from Request V1.1.
/// Instead, this type should be converted to/from an AriesMessage
pub type Request = MsgParts<RequestContent, RequestDecorators>;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use diddoc_legacy::aries::diddoc::AriesDidDoc;
    use serde_json::json;
    use shared::maybe_known::MaybeKnown;

    use super::*;
    use crate::{
        decorators::{
            attachment::{Attachment, AttachmentData, AttachmentType},
            thread::{tests::make_extended_thread, ThreadGoalCode},
            timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_fields::protocols::did_exchange::{
            v1_0::request::{Request, RequestDecorators},
            v1_x::request::AnyRequest,
        },
        msg_types::protocols::did_exchange::DidExchangeTypeV1_0,
    };

    pub fn request_content() -> RequestContent {
        let did_doc = AriesDidDoc::default();
        RequestContent {
            label: "test_request_label".to_owned(),
            goal_code: Some(MaybeKnown::Known(ThreadGoalCode::AriesRelBuild)),
            goal: Some("test_goal".to_owned()),
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
    fn test_print_message() {
        let msg: Request = Request::builder()
            .id("test_id".to_owned())
            .content(request_content())
            .decorators(RequestDecorators::default())
            .build();
        let printed_json = format!("{}", msg);
        let parsed_request: Request = serde_json::from_str(&printed_json).unwrap();
        assert_eq!(msg, parsed_request);
    }

    #[test]
    fn test_minimal_didexchange_request() {
        let content = request_content();
        let expected = json!({
            "label": content.label,
            "goal_code": content.goal_code,
            "goal": content.goal,
            "did": content.did,
            "did_doc~attach": content.did_doc,
        });

        let msg = AnyRequest::V1_0(
            Request::builder()
                .id("test".to_owned())
                .content(content)
                .decorators(RequestDecorators::default())
                .build(),
        );

        test_utils::test_constructed_msg(msg, DidExchangeTypeV1_0::Request, expected);
    }

    #[test]
    fn test_extended_didexchange_request() {
        let content = request_content();

        let mut decorators = RequestDecorators::default();
        decorators.thread = Some(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "label": content.label,
            "goal_code": content.goal_code,
            "goal": content.goal,
            "did": content.did,
            "did_doc~attach": content.did_doc,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        let msg = AnyRequest::V1_0(
            Request::builder()
                .id("test".to_owned())
                .content(content)
                .decorators(decorators)
                .build(),
        );

        test_utils::test_constructed_msg(msg, DidExchangeTypeV1_0::Request, expected);
    }
}
