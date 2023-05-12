use serde::{Deserialize, Serialize};

use crate::{
    msg_fields::protocols::notification::ack::{AckContent, AckDecorators, AckStatus},
    msg_parts::MsgParts,
};

pub type AckPresentation = MsgParts<AckPresentationContent, AckDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub struct AckPresentationContent(pub AckContent);

impl AckPresentationContent {
    pub fn new(status: AckStatus) -> Self {
        Self(AckContent::new(status))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{thread::tests::make_extended_thread, timing::tests::make_extended_timing},
        misc::test_utils,
        msg_types::present_proof::PresentProofTypeV1_0,
    };

    #[test]
    fn test_minimal_ack_proof() {
        let content = AckPresentationContent::new(AckStatus::Ok);

        let decorators = AckDecorators::new(make_extended_thread());

        let expected = json!({
            "status": content.0.status,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, PresentProofTypeV1_0::Ack, expected);
    }

    #[test]
    fn test_extended_ack_proof() {
        let content = AckPresentationContent::new(AckStatus::Ok);

        let mut decorators = AckDecorators::new(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "status": content.0.status,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, PresentProofTypeV1_0::Ack, expected);
    }
}
