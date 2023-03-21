use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    message::Message,
    msg_types::types::present_proof::PresentProofV1_0,
    protocols::notification::{AckContent, AckDecorators, AckStatus},
};

pub type AckPresentation = Message<AckPresentationContent, AckDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "PresentProofV1_0::Ack")]
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
    };

    #[test]
    fn test_minimal_ack_proof() {
        let content = AckPresentationContent::new(AckStatus::Ok);

        let decorators = AckDecorators::new(make_extended_thread());

        let json = json!({
            "status": content.0.status,
            "~thread": decorators.thread
        });

        test_utils::test_msg::<AckPresentationContent, _, _>(content, decorators, json);
    }

    #[test]
    fn test_extensive_ack_proof() {
        let content = AckPresentationContent::new(AckStatus::Ok);

        let mut decorators = AckDecorators::new(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let json = json!({
            "status": content.0.status,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg::<AckPresentationContent, _, _>(content, decorators, json);
    }
}
