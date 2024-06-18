use crate::{
    msg_fields::protocols::did_exchange::v1_x::request::{RequestContent, RequestDecorators},
    msg_parts::MsgParts,
    msg_types::{protocols::did_exchange::DidExchangeTypeV1_0, MsgKindType},
};

pub type RequestContentV1_0 = RequestContent<MsgKindType<DidExchangeTypeV1_0>>;
pub type Request = MsgParts<RequestContentV1_0, RequestDecorators>;

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
        msg_fields::protocols::did_exchange::v1_0::request::{Request, RequestDecorators},
        msg_types::protocols::did_exchange::DidExchangeTypeV1_0,
    };

    pub fn request_content() -> RequestContentV1_0 {
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
            _marker: std::marker::PhantomData,
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
        test_utils::test_msg(
            content,
            RequestDecorators::default(),
            DidExchangeTypeV1_0::Request,
            expected,
        );
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

        test_utils::test_msg(content, decorators, DidExchangeTypeV1_0::Request, expected);
    }
}
