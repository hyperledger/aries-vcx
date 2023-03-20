use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    msg_types::types::revocation::RevocationV2_0Kind,
    protocols::notification::{AckContent, AckDecorators, AckStatus},
    Message,
};

pub type AckRevoke = Message<AckRevokeContent, AckDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "RevocationV2_0Kind::Ack")]
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
        misc::test_utils, decorators::{thread::tests::make_extended_thread, timing::tests::make_extended_timing},
    };

    #[test]
    fn test_minimal_ack_revoke() {
        let msg_type = test_utils::build_msg_type::<AckRevokeContent>();

        let status = AckStatus::Ok;
        let content = AckRevokeContent::new(status);

        let thread = make_extended_thread();
        let decorators = AckDecorators::new(thread);

        let json = json!({
            "@type": msg_type,
            "status": status,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, json);
    }

    #[test]
    fn test_extensive_ack_revoke() {
        let msg_type = test_utils::build_msg_type::<AckRevokeContent>();

        let status = AckStatus::Ok;
        let content = AckRevokeContent::new(status);

        let thread = make_extended_thread();
        let mut decorators = AckDecorators::new(thread);
        let timing = make_extended_timing();
        decorators.timing = Some(timing);

        let json = json!({
            "@type": msg_type,
            "status": status,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, json);
    }
}
