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
        decorators::{Thread, Timing},
        misc::test_utils,
    };

    #[test]
    fn test_minimal_message() {
        let msg_type = test_utils::build_msg_type::<AckRevokeContent>();

        let status = AckStatus::Ok;
        let content = AckRevokeContent::new(status);

        let thid = "test".to_owned();
        let thread = Thread::new(thid.clone());
        let decorators = AckDecorators::new(thread);

        let json = json!({
            "@type": msg_type,
            "status": status,
            "~thread": {
                "thid": thid
            }
        });

        test_utils::test_msg(content, decorators, json);
    }

    #[test]
    fn test_extensive_message() {
        let msg_type = test_utils::build_msg_type::<AckRevokeContent>();

        let status = AckStatus::Ok;
        let content = AckRevokeContent::new(status);

        let thid = "test".to_owned();
        let thread = Thread::new(thid.clone());
        let mut decorators = AckDecorators::new(thread);
        let in_time = "test".to_owned();
        let mut timing = Timing::default();
        timing.in_time = Some(in_time.clone());
        decorators.timing = Some(timing);

        let json = json!({
            "@type": msg_type,
            "status": status,
            "~thread": {
                "thid": thid
            },
            "~timing": {
                "in_time": in_time
            }
        });

        test_utils::test_msg(content, decorators, json);
    }
}
