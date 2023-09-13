use serde::{Deserialize, Serialize};
use shared_vcx::maybe_known::MaybeKnown;
use typed_builder::TypedBuilder;

use crate::{
    decorators::{please_ack::PleaseAck, thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type Revoke = MsgParts<RevokeContent, RevokeDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]

pub struct RevokeContent {
    pub credential_id: String,
    pub revocation_format: MaybeKnown<RevocationFormat>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct RevokeDecorators {
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum RevocationFormat {
    IndyAnoncreds,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::thread::tests::make_extended_thread, misc::test_utils, msg_types::revocation::RevocationTypeV2_0,
    };

    #[test]
    fn test_minimal_revoke() {
        let content = RevokeContent::builder()
            .credential_id("test_credential_id".to_owned())
            .revocation_format(MaybeKnown::Known(RevocationFormat::IndyAnoncreds))
            .build();

        let decorators = RevokeDecorators::default();

        let expected = json!({
            "credential_id": content.credential_id,
            "revocation_format": content.revocation_format
        });

        test_utils::test_msg(content, decorators, RevocationTypeV2_0::Revoke, expected);
    }

    #[test]
    fn test_extended_revoke() {
        let content = RevokeContent::builder()
            .credential_id("test_credential_id".to_owned())
            .revocation_format(MaybeKnown::Known(RevocationFormat::IndyAnoncreds))
            .comment("test_comment".to_owned())
            .build();

        let decorators = RevokeDecorators::builder().thread(make_extended_thread()).build();

        let expected = json!({
            "credential_id": content.credential_id,
            "revocation_format": content.revocation_format,
            "comment": content.comment,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, RevocationTypeV2_0::Revoke, expected);
    }
}
