use serde::{Deserialize, Serialize};
use shared::maybe_known::MaybeKnown;
use typed_builder::TypedBuilder;

use crate::{
    decorators::{
        attachment::Attachment,
        thread::{Thread, ThreadGoalCode},
        timing::Timing,
    },
    msg_parts::MsgParts,
};

pub type Request = MsgParts<RequestContent, RequestDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct RequestContent {
    pub label: String,
    pub goal_code: Option<MaybeKnown<ThreadGoalCode>>,
    pub goal: Option<String>,
    pub did: String, // TODO: Use Did
    #[serde(rename = "did_doc~attach")]
    pub did_doc: Option<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct RequestDecorators {
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
pub mod tests {
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
