use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    msg_fields::protocols::notification::ack::{Ack, AckContent, AckDecorators},
    msg_parts::MsgParts,
};

pub type AckPresentationV1 = MsgParts<AckPresentationV1Content, AckDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
#[serde(transparent)]
pub struct AckPresentationV1Content {
    pub inner: AckContent,
}

impl From<AckContent> for AckPresentationV1Content {
    fn from(value: AckContent) -> Self {
        Self { inner: value }
    }
}

impl From<AckPresentationV1> for Ack {
    fn from(value: AckPresentationV1) -> Self {
        Self::builder()
            .id(value.id)
            .content(value.content.inner)
            .decorators(value.decorators)
            .build()
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{thread::tests::make_extended_thread, timing::tests::make_extended_timing},
        misc::test_utils,
        msg_fields::protocols::notification::ack::AckStatus,
        msg_types::present_proof::PresentProofTypeV1_0,
    };

    #[test]
    fn test_minimal_ack_proof() {
        let content: AckPresentationV1Content = AckContent::builder().status(AckStatus::Ok).build();

        let decorators = AckDecorators::builder()
            .thread(make_extended_thread())
            .build();

        let expected = json!({
            "status": content.inner.status,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, PresentProofTypeV1_0::Ack, expected);
    }

    #[test]
    fn test_extended_ack_proof() {
        let content: AckPresentationV1Content = AckContent::builder().status(AckStatus::Ok).build();

        let decorators = AckDecorators::builder()
            .thread(make_extended_thread())
            .timing(make_extended_timing())
            .build();

        let expected = json!({
            "status": content.inner.status,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, PresentProofTypeV1_0::Ack, expected);
    }
}
