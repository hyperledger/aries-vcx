use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{attachment::Attachment, please_ack::PleaseAck, thread::Thread, timing::Timing},
    msg_fields::protocols::common::attachment_format_specifier::AttachmentFormatSpecifier,
    msg_parts::MsgParts,
};

pub type PresentationV2 = MsgParts<PresentationV2Content, PresentationV2Decorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct PresentationV2Content {
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<String>,
    pub formats: Vec<AttachmentFormatSpecifier<PresentationAttachmentFormatType>>,
    #[serde(rename = "presentations~attach")]
    pub presentations_attach: Vec<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct PresentationV2Decorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

/// Format types derived from Aries RFC Registry:
/// https://github.com/hyperledger/aries-rfcs/tree/b3a3942ef052039e73cd23d847f42947f8287da2/features/0454-present-proof-v2#presentations-attachment-registry
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum PresentationAttachmentFormatType {
    #[serde(rename = "hlindy/proof@v2.0")]
    HyperledgerIndyProof2_0,
    #[serde(rename = "anoncreds/proof@v1.0")]
    AnoncredsProof1_0,
    #[serde(rename = "dif/presentation-exchange/submission@v1.0")]
    DifPresentationExchangeSubmission1_0,
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;
    use shared::maybe_known::MaybeKnown;

    use super::*;
    use crate::{
        decorators::{
            attachment::tests::make_extended_attachment,
            please_ack::tests::make_minimal_please_ack, thread::tests::make_extended_thread,
            timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_types::present_proof::PresentProofTypeV2_0,
    };

    #[test]
    fn test_minimal_present_proof() {
        let content = PresentationV2Content::builder()
            .formats(vec![AttachmentFormatSpecifier {
                attach_id: "1".to_owned(),
                format: MaybeKnown::Known(
                    PresentationAttachmentFormatType::HyperledgerIndyProof2_0,
                ),
            }])
            .presentations_attach(vec![make_extended_attachment()])
            .build();

        let decorators = PresentationV2Decorators::builder()
            .thread(make_extended_thread())
            .build();

        let expected = json!({
            "formats": content.formats,
            "presentations~attach": content.presentations_attach,
            "~thread": decorators.thread
        });

        test_utils::test_msg(
            content,
            decorators,
            PresentProofTypeV2_0::Presentation,
            expected,
        );
    }

    #[test]
    fn test_extended_present_proof() {
        let content = PresentationV2Content::builder()
            .formats(vec![AttachmentFormatSpecifier {
                attach_id: "1".to_owned(),
                format: MaybeKnown::Known(
                    PresentationAttachmentFormatType::HyperledgerIndyProof2_0,
                ),
            }])
            .presentations_attach(vec![make_extended_attachment()])
            .comment(Some("test_comment".to_owned()))
            .goal_code(Some("goal.goal".to_owned()))
            .build();

        let decorators = PresentationV2Decorators::builder()
            .thread(make_extended_thread())
            .timing(make_extended_timing())
            .please_ack(make_minimal_please_ack())
            .build();

        let expected = json!({
            "comment": content.comment,
            "goal_code": content.goal_code,
            "formats": content.formats,
            "presentations~attach": content.presentations_attach,
            "~thread": decorators.thread,
            "~timing": decorators.timing,
            "~please_ack": decorators.please_ack
        });

        test_utils::test_msg(
            content,
            decorators,
            PresentProofTypeV2_0::Presentation,
            expected,
        );
    }
}
