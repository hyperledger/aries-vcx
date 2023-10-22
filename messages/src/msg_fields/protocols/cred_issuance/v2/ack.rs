use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    msg_fields::protocols::notification::ack::{Ack, AckContent, AckDecorators},
    msg_parts::MsgParts,
};

pub type AckCredentialV2 = MsgParts<AckCredentialV2Content, AckDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
#[serde(transparent)]
pub struct AckCredentialV2Content {
    pub inner: AckContent,
}

impl From<AckContent> for AckCredentialV2Content {
    fn from(value: AckContent) -> Self {
        Self { inner: value }
    }
}

impl From<AckCredentialV2> for Ack {
    fn from(value: AckCredentialV2) -> Self {
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
        msg_types::cred_issuance::CredentialIssuanceTypeV2_0,
    };

    #[test]
    fn test_minimal_ack_cred() {
        let content: AckCredentialV2Content = AckContent::builder().status(AckStatus::Ok).build();

        let decorators = AckDecorators::builder()
            .thread(make_extended_thread())
            .build();

        let expected = json!({
            "status": content.inner.status,
            "~thread": decorators.thread
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV2_0::Ack,
            expected,
        );
    }

    #[test]
    fn test_extended_ack_cred() {
        let content: AckCredentialV2Content = AckContent::builder().status(AckStatus::Ok).build();

        let decorators = AckDecorators::builder()
            .thread(make_extended_thread())
            .timing(make_extended_timing())
            .build();

        let expected = json!({
            "status": content.inner.status,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV2_0::Ack,
            expected,
        );
    }
}
