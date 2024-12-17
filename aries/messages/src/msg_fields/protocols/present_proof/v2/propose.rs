use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{attachment::Attachment, thread::Thread, timing::Timing},
    msg_fields::protocols::common::attachment_format_specifier::OptionalIdAttachmentFormatSpecifier,
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
    #[serde(rename = "anoncreds/proof-request@v1.0")]
    AnoncredsProofRequest1_0,
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;
    use shared::maybe_known::MaybeKnown;

    use super::*;
    use crate::{
        decorators::{
            attachment::tests::make_extended_attachment, thread::tests::make_extended_thread,
            timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_types::present_proof::PresentProofTypeV2_0,
    };

    #[test]
    fn test_minimal_propose_proof() {
        let content = ProposePresentationV2Content::builder()
            .formats(vec![OptionalIdAttachmentFormatSpecifier {
                attach_id: None,
                format: MaybeKnown::Known(
                    ProposePresentationAttachmentFormatType::HyperledgerIndyProofRequest2_0,
                ),
            }])
            .proposals_attach(None)
            .build();

        let decorators = ProposePresentationV2Decorators::default();

        let expected = json!({
            "formats": content.formats
        });

        test_utils::test_msg(
            content,
            decorators,
            PresentProofTypeV2_0::ProposePresentation,
            expected,
        );
    }

    #[test]
    fn test_extended_propose_proof() {
        let content = ProposePresentationV2Content::builder()
            .formats(vec![OptionalIdAttachmentFormatSpecifier {
                attach_id: Some("1".to_owned()),
                format: MaybeKnown::Known(
                    ProposePresentationAttachmentFormatType::HyperledgerIndyProofRequest2_0,
                ),
            }])
            .proposals_attach(Some(vec![make_extended_attachment()]))
            .goal_code(Some("goal.goal".to_owned()))
            .comment(Some("test_comment".to_owned()))
            .build();

        let decorators = ProposePresentationV2Decorators::builder()
            .thread(Some(make_extended_thread()))
            .timing(Some(make_extended_timing()))
            .build();

        let expected = json!({
            "comment": content.comment,
            "goal_code": content.goal_code,
            "formats": content.formats,
            "proposals~attach": content.proposals_attach,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(
            content,
            decorators,
            PresentProofTypeV2_0::ProposePresentation,
            expected,
        );
    }
}
