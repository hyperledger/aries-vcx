use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    msg_types::types::cred_issuance::CredentialIssuanceV1_0Kind,
    protocols::notification::{AckContent, AckDecorators, AckStatus},
    Message,
};

pub type AckCredential = Message<AckCredentialContent, AckDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "CredentialIssuanceV1_0Kind::Ack")]
#[serde(transparent)]
pub struct AckCredentialContent(pub AckContent);

impl AckCredentialContent {
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
    fn test_minimal_ack_cred() {
        let msg_type = test_utils::build_msg_type::<AckCredentialContent>();

        let content = AckCredentialContent::new(AckStatus::Ok);

        let decorators = AckDecorators::new(make_extended_thread());

        let json = json!({
            "@type": msg_type,
            "status": content.0.status,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, json);
    }

    #[test]
    fn test_extensive_ack_cred() {
        let msg_type = test_utils::build_msg_type::<AckCredentialContent>();

        let content = AckCredentialContent::new(AckStatus::Ok);

        let mut decorators = AckDecorators::new(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let json = json!({
            "@type": msg_type,
            "status": content.0.status,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, json);
    }
}
