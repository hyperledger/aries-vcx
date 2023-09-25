use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{attachment::Attachment, please_ack::PleaseAck, thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type Presentation = MsgParts<PresentationContent, PresentationDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct PresentationContent {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "presentations~attach")]
    pub presentations_attach: Vec<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct PresentationDecorators {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{
            attachment::tests::make_extended_attachment,
            please_ack::tests::make_minimal_please_ack, thread::tests::make_extended_thread,
            timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_types::present_proof::PresentProofTypeV1_0,
    };

    #[test]
    fn test_minimal_present_proof() {
        let content = PresentationContent::builder()
            .presentations_attach(vec![make_extended_attachment()])
            .build();

        let decorators = PresentationDecorators::builder()
            .thread(make_extended_thread())
            .build();

        let expected = json!({
            "presentations~attach": content.presentations_attach,
            "~thread": decorators.thread
        });

        test_utils::test_msg(
            content,
            decorators,
            PresentProofTypeV1_0::Presentation,
            expected,
        );
    }

    #[test]
    fn test_extended_present_proof() {
        let content = PresentationContent::builder()
            .presentations_attach(vec![make_extended_attachment()])
            .comment("test_comment".to_owned())
            .build();

        let decorators = PresentationDecorators::builder()
            .thread(make_extended_thread())
            .timing(make_extended_timing())
            .please_ack(make_minimal_please_ack())
            .build();

        let expected = json!({
            "comment": content.comment,
            "presentations~attach": content.presentations_attach,
            "~thread": decorators.thread,
            "~timing": decorators.timing,
            "~please_ack": decorators.please_ack
        });

        test_utils::test_msg(
            content,
            decorators,
            PresentProofTypeV1_0::Presentation,
            expected,
        );
    }
}
