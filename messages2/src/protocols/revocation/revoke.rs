use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{PleaseAck, Thread, Timing},
    msg_types::types::revocation::RevocationV2_0Kind,
    Message,
};

pub type Revoke = Message<RevokeContent, RevokeDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "RevocationV2_0Kind::Revoke")]
pub struct RevokeContent {
    pub credential_id: String,
    pub revocation_format: RevocationFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl RevokeContent {
    pub fn new(credential_id: String, revocation_format: RevocationFormat) -> Self {
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
    use crate::misc::test_utils;

    #[test]
    fn test_minimal_revoke() {
        let msg_type = test_utils::build_msg_type::<RevokeContent>();

        let credential_id = "test".to_owned();
        let format = RevocationFormat::IndyAnoncreds;
        let content = RevokeContent::new(credential_id.clone(), format);

        let decorators = RevokeDecorators::default();

        let json = json!({
            "@type": msg_type,
            "credential_id": credential_id,
            "revocation_format": format
        });

        test_utils::test_msg(content, decorators, json);
    }

    #[test]
    fn test_extensive_revoke() {
        let msg_type = test_utils::build_msg_type::<RevokeContent>();

        let credential_id = "test".to_owned();
        let format = RevocationFormat::IndyAnoncreds;
        let mut content = RevokeContent::new(credential_id.clone(), format);
        let comment = "test".to_owned();
        content.comment = Some(comment.clone());

        let mut decorators = RevokeDecorators::default();
        let thid = "test".to_owned();
        let thread = Thread::new(thid.clone());
        decorators.thread = Some(thread);

        let json = json!({
            "@type": msg_type,
            "credential_id": credential_id,
            "revocation_format": format,
            "comment": comment,
            "~thread": {
                "thid": thid
            }
        });

        test_utils::test_msg(content, decorators, json);
    }
}
