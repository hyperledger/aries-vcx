use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{
        attachment::{Attachment, AttachmentFormatSpecifier},
        thread::Thread,
        timing::Timing,
    },
    msg_parts::MsgParts,
};

pub type RequestPresentationV2 =
    MsgParts<RequestPresentationV2Content, RequestPresentationV2Decorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct RequestPresentationV2Content {
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<String>, // TODO - spec does not specify what goal codes to use..
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_confirm: Option<bool>,
    pub formats: Vec<AttachmentFormatSpecifier<PresentationRequestAttachmentFormatType>>,
    #[serde(rename = "request_presentations~attach")]
    pub request_presentations_attach: Vec<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct RequestPresentationV2Decorators {
    #[builder(default)]
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[builder(default)]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

/// Format types derived from Aries RFC Registry:
/// https://github.com/hyperledger/aries-rfcs/tree/b3a3942ef052039e73cd23d847f42947f8287da2/features/0454-present-proof-v2#presentation-request-attachment-registry
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum PresentationRequestAttachmentFormatType {
    #[serde(rename = "hlindy/proof-req@v2.0")]
    HyperledgerIndyProofRequest2_0,
    #[serde(rename = "dif/presentation-exchange/definitions@v1.0")]
    DifPresentationExchangeDefinitions1_0,
}

// #[cfg(test)]
// #[allow(clippy::unwrap_used)]
// #[allow(clippy::field_reassign_with_default)]
// mod tests {
//     use serde_json::json;

//     use super::*;
//     use crate::{
//         decorators::{
//             attachment::tests::make_extended_attachment, thread::tests::make_extended_thread,
//             timing::tests::make_extended_timing,
//         },
//         misc::test_utils,
//         msg_types::present_proof::PresentProofTypeV1_0,
//     };

//     #[test]
//     fn test_minimal_request_proof() {
//         let content = RequestPresentationV2Content::builder()
//             .request_presentations_attach(vec![make_extended_attachment()])
//             .build();

//         let decorators = RequestPresentationV2Decorators::default();

//         let expected = json!({
//             "request_presentations~attach": content.request_presentations_attach,
//         });

//         test_utils::test_msg(
//             content,
//             decorators,
//             PresentProofTypeV1_0::RequestPresentation,
//             expected,
//         );
//     }

//     #[test]
//     fn test_extended_request_proof() {
//         let content = RequestPresentationV2Content::builder()
//             .request_presentations_attach(vec![make_extended_attachment()])
//             .comment("test_comment".to_owned())
//             .build();

//         let decorators = RequestPresentationV2Decorators::builder()
//             .thread(make_extended_thread())
//             .timing(make_extended_timing())
//             .build();

//         let expected = json!({
//             "request_presentations~attach": content.request_presentations_attach,
//             "comment": content.comment,
//             "~thread": decorators.thread,
//             "~timing": decorators.timing
//         });

//         test_utils::test_msg(
//             content,
//             decorators,
//             PresentProofTypeV1_0::RequestPresentation,
//             expected,
//         );
//     }
// }
