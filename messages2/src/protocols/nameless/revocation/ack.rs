use serde::{Deserialize, Serialize};

use crate::{
    msg_parts::MsgParts,
    protocols::nameless::notification::{AckContent, AckDecorators, AckStatus},
};

pub type AckRevoke = MsgParts<AckRevokeContent, AckDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub struct AckRevokeContent(pub AckContent);

impl AckRevokeContent {
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
        misc::test_utils, msg_types::revocation::RevocationProtocolV2_0,
    };

    #[test]
    fn test_minimal_ack_revoke() {
        let content = AckRevokeContent::new(AckStatus::Ok);

        let decorators = AckDecorators::new(make_extended_thread());

        let expected = json!({
            "status": content.0.status,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, RevocationProtocolV2_0::Ack, expected);
    }

    #[test]
    fn test_extended_ack_revoke() {
        let content = AckRevokeContent::new(AckStatus::Ok);

        let mut decorators = AckDecorators::new(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "status": content.0.status,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, RevocationProtocolV2_0::Ack, expected);
    }
}
