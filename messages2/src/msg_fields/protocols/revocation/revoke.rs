use serde::{Deserialize, Serialize};

use crate::{
    decorators::{please_ack::PleaseAck, thread::Thread, timing::Timing},
    maybe_known::MaybeKnown,
    msg_parts::MsgParts,
};

pub type Revoke = MsgParts<RevokeContent, RevokeDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]

pub struct RevokeContent {
    pub credential_id: String,
    pub revocation_format: MaybeKnown<RevocationFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl RevokeContent {
    pub fn new(credential_id: String, revocation_format: MaybeKnown<RevocationFormat>) -> Self {
        Self {
            credential_id,
            revocation_format,
            comment: None,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct RevokeDecorators {
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
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
        let content = RevokeContent::new(
            "test_credential_id".to_owned(),
            MaybeKnown::Known(RevocationFormat::IndyAnoncreds),
        );

        let decorators = RevokeDecorators::default();

        let expected = json!({
            "credential_id": content.credential_id,
            "revocation_format": content.revocation_format
        });

        test_utils::test_msg(content, decorators, RevocationTypeV2_0::Revoke, expected);
    }

    #[test]
    fn test_extended_revoke() {
        let mut content = RevokeContent::new(
            "test_credential_id".to_owned(),
            MaybeKnown::Known(RevocationFormat::IndyAnoncreds),
        );
        content.comment = Some("test_comment".to_owned());

        let mut decorators = RevokeDecorators::default();
        decorators.thread = Some(make_extended_thread());

        let expected = json!({
            "credential_id": content.credential_id,
            "revocation_format": content.revocation_format,
            "comment": content.comment,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, RevocationTypeV2_0::Revoke, expected);
    }
}
