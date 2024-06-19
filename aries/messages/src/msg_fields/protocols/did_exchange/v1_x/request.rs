use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use shared::maybe_known::MaybeKnown;
use typed_builder::TypedBuilder;

use crate::{
    decorators::{
        attachment::Attachment,
        thread::{Thread, ThreadGoalCode},
        timing::Timing,
    },
    msg_fields::protocols::did_exchange::{
        v1_0::{request::Request as RequestV1_0, DidExchangeV1_0},
        v1_1::{
            request::{Request as RequestV1_1, RequestContentV1_1},
            DidExchangeV1_1,
        },
        DidExchange,
    },
    msg_parts::MsgParts,
    msg_types::protocols::did_exchange::DidExchangeTypeV1,
    AriesMessage,
};

pub type Request<MinorVer> = MsgParts<RequestContent<MinorVer>, RequestDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct RequestContent<MinorVer> {
    pub label: String,
    pub goal_code: Option<MaybeKnown<ThreadGoalCode>>,
    pub goal: Option<String>,
    pub did: String, // TODO: Use Did
    #[serde(rename = "did_doc~attach")]
    pub did_doc: Option<Attachment>,
    #[builder(default, setter(skip))]
    #[serde(skip)]
    pub(crate) _marker: PhantomData<MinorVer>,
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, derive_more::From)]
#[serde(untagged)]
pub enum AnyRequest {
    V1_0(RequestV1_0),
    V1_1(RequestV1_1),
}

impl AnyRequest {
    pub fn get_version_marker(&self) -> DidExchangeTypeV1 {
        match self {
            AnyRequest::V1_0(_) => DidExchangeTypeV1::new_v1_0(),
            AnyRequest::V1_1(_) => DidExchangeTypeV1::new_v1_1(),
        }
    }

    pub fn into_v1_1(self) -> RequestV1_1 {
        match self {
            AnyRequest::V1_0(r) => r.into_v1_1(),
            AnyRequest::V1_1(r) => r,
        }
    }
}

impl RequestV1_0 {
    pub fn into_v1_1(self) -> RequestV1_1 {
        RequestV1_1 {
            id: self.id,
            content: RequestContentV1_1 {
                label: self.content.label,
                goal_code: self.content.goal_code,
                goal: self.content.goal,
                did: self.content.did,
                did_doc: self.content.did_doc,
                _marker: PhantomData,
            },
            decorators: self.decorators,
        }
    }
}

impl From<AnyRequest> for AriesMessage {
    fn from(value: AnyRequest) -> Self {
        match value {
            AnyRequest::V1_0(inner) => DidExchange::V1_0(DidExchangeV1_0::Request(inner)).into(),
            AnyRequest::V1_1(inner) => DidExchange::V1_1(DidExchangeV1_1::Request(inner)).into(),
        }
    }
}

// #[cfg(test)]
// #[allow(clippy::unwrap_used)]
// #[allow(clippy::field_reassign_with_default)]
// mod tests {
//     use diddoc_legacy::aries::diddoc::AriesDidDoc;
//     use serde_json::json;

//     use super::*;
//     use crate::{
//         decorators::{
//             attachment::{AttachmentData, AttachmentType},
//             thread::tests::make_extended_thread,
//             timing::tests::make_extended_timing,
//         },
//         misc::test_utils,
//         msg_types::protocols::did_exchange::DidExchangeTypeV1_0,
//     };

//     pub fn request_content() -> RequestContent<()> {
//         let did_doc = AriesDidDoc::default();
//         RequestContent {
//             label: "test_request_label".to_owned(),
//             goal_code: Some(MaybeKnown::Known(ThreadGoalCode::AriesRelBuild)),
//             goal: Some("test_goal".to_owned()),
//             did: did_doc.id.clone(),
//             did_doc: Some(
//                 Attachment::builder()
//                     .data(
//                         AttachmentData::builder()
//                             .content(AttachmentType::Json(
//                                 serde_json::to_value(&did_doc).unwrap(),
//                             ))
//                             .build(),
//                     )
//                     .build(),
//             ),
//             _marker: PhantomData,
//         }
//     }

//     #[test]
//     fn test_print_message() {
//         let msg: Request<_> = Request::<()>::builder()
//             .id("test_id".to_owned())
//             .content(request_content())
//             .decorators(RequestDecorators::default())
//             .build();
//         let printed_json = format!("{}", msg);
//         let parsed_request: Request<_> = serde_json::from_str(&printed_json).unwrap();
//         assert_eq!(msg, parsed_request);
//     }

//     #[test]
//     fn test_minimal_didexchange_request() {
//         let content = request_content();
//         let expected = json!({
//             "label": content.label,
//             "goal_code": content.goal_code,
//             "goal": content.goal,
//             "did": content.did,
//             "did_doc~attach": content.did_doc,
//         });
//         test_utils::test_msg(
//             content,
//             RequestDecorators::default(),
//             DidExchangeTypeV1_0::Request,
//             expected,
//         );
//     }

//     #[test]
//     fn test_extended_didexchange_request() {
//         let content = request_content();

//         let mut decorators = RequestDecorators::default();
//         decorators.thread = Some(make_extended_thread());
//         decorators.timing = Some(make_extended_timing());

//         let expected = json!({
//             "label": content.label,
//             "goal_code": content.goal_code,
//             "goal": content.goal,
//             "did": content.did,
//             "did_doc~attach": content.did_doc,
//             "~thread": decorators.thread,
//             "~timing": decorators.timing
//         });

//         test_utils::test_msg(content, decorators, DidExchangeTypeV1_0::Request, expected);
//     }
// }
