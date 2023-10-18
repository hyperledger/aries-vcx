use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{
        attachment::{Attachment, OptionalIdAttachmentFormatSpecifier},
        thread::Thread,
        timing::Timing,
    },
    msg_parts::MsgParts,
};

pub type ProposePresentationV2 =
    MsgParts<ProposePresentationV2Content, ProposePresentationV2Decorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct ProposePresentationV2Content {
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<String>, // TODO - spec does not specify what goal codes to use..
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub formats: Vec<OptionalIdAttachmentFormatSpecifier<ProposePresentationAttachmentFormatType>>,
    #[serde(rename = "proposals~attach", skip_serializing_if = "Option::is_none")]
    pub proposals_attach: Option<Vec<Attachment>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct ProposePresentationV2Decorators {
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
/// https://github.com/hyperledger/aries-rfcs/tree/b3a3942ef052039e73cd23d847f42947f8287da2/features/0454-present-proof-v2#propose-attachment-registry
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum ProposePresentationAttachmentFormatType {
    #[serde(rename = "dif/presentation-exchange/definitions@v1.0")]
    DifPresentationExchangeDefinitions1_0,
    #[serde(rename = "hlindy/proof-req@v2.0")]
    HyperledgerIndyProofRequest2_0,
}

// #[cfg(test)]
// #[allow(clippy::unwrap_used)]
// #[allow(clippy::field_reassign_with_default)]
// mod tests {
//     use serde_json::json;

//     use super::*;
//     use crate::{
//         decorators::{thread::tests::make_extended_thread, timing::tests::make_extended_timing},
//         misc::test_utils,
//     };

//     #[test]
//     fn test_minimal_propose_proof() {
//         let attribute = PresentationAttr::builder()
//             .name("test_attribute_name".to_owned())
//             .build();
//         let predicate = Predicate::builder()
//             .name("test_predicate_name".to_owned())
//             .predicate(PredicateOperator::GreaterOrEqual)
//             .threshold(1000)
//             .build();
//         let preview = PresentationPreview::new(vec![attribute], vec![predicate]);
//         let content = ProposePresentationV2Content::builder()
//             .presentation_proposal(preview)
//             .build();

//         let decorators = ProposePresentationV2Decorators::default();

//         let expected = json!({
//             "presentation_proposal": content.presentation_proposal
//         });

//         test_utils::test_msg(
//             content,
//             decorators,
//             PresentProofTypeV1_0::ProposePresentation,
//             expected,
//         );
//     }

//     #[test]
//     fn test_extended_propose_proof() {
//         let attribute = PresentationAttr::builder()
//             .name("test_attribute_name".to_owned())
//             .build();
//         let predicate = Predicate::builder()
//             .name("test_predicate_name".to_owned())
//             .predicate(PredicateOperator::GreaterOrEqual)
//             .threshold(1000)
//             .build();
//         let preview = PresentationPreview::new(vec![attribute], vec![predicate]);
//         let content = ProposePresentationV2Content::builder()
//             .presentation_proposal(preview)
//             .comment("test_comment".to_owned())
//             .build();

//         let decorators = ProposePresentationV2Decorators::builder()
//             .thread(make_extended_thread())
//             .timing(make_extended_timing())
//             .build();

//         let expected = json!({
//             "comment": content.comment,
//             "presentation_proposal": content.presentation_proposal,
//             "~thread": decorators.thread,
//             "~timing": decorators.timing
//         });

//         test_utils::test_msg(
//             content,
//             decorators,
//             PresentProofTypeV1_0::ProposePresentation,
//             expected,
//         );
//     }
// }
