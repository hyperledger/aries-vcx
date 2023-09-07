use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    msg_fields::protocols::notification::ack::{Ack, AckContent, AckDecorators},
    msg_parts::MsgParts,
};

pub type AckRevoke = MsgParts<AckRevokeContent, AckDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
#[serde(transparent)]
pub struct AckRevokeContent {
    pub inner: AckContent,
}

impl From<AckContent> for AckRevokeContent {
    fn from(value: AckContent) -> Self {
        Self { inner: value }
    }
}

impl From<AckRevoke> for Ack {
    fn from(value: AckRevoke) -> Self {
        Self::builder()
            .id(value.id)
            .content(value.content.inner)
            .decorators(value.decorators)
            .build()
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
        msg_fields::protocols::notification::ack::AckStatus,
        msg_types::revocation::RevocationTypeV2_0,
    };

    #[test]
    fn test_minimal_ack_revoke() {
        let content: AckRevokeContent = AckContent::builder().status(AckStatus::Ok).build();

        let decorators = AckDecorators::builder().thread(make_extended_thread()).build();

        let expected = json!({
            "status": content.inner.status,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, RevocationTypeV2_0::Ack, expected);
    }

    #[test]
    fn test_extended_ack_revoke() {
        let content: AckRevokeContent = AckContent::builder().status(AckStatus::Ok).build();

        let mut decorators = AckDecorators::builder().thread(make_extended_thread()).build();
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "status": content.inner.status,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, RevocationTypeV2_0::Ack, expected);
    }
}
