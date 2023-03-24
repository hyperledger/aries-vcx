use serde::{Deserialize, Serialize};

use crate::{
    decorators::{attachment::Attachment, please_ack::PleaseAck, thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type IssueCredential = MsgParts<IssueCredentialContent, IssueCredentialDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct IssueCredentialContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "credentials~attach")]
    pub credentials_attach: Vec<Attachment>,
}

impl IssueCredentialContent {
    pub fn new(credentials_attach: Vec<Attachment>) -> Self {
        Self {
            comment: None,
            credentials_attach,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct IssueCredentialDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

impl IssueCredentialDecorators {
    pub fn new(thread: Thread) -> Self {
        Self {
            thread,
            please_ack: None,
            timing: None,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{
            attachment::tests::make_extended_attachment, please_ack::tests::make_minimal_please_ack,
            thread::tests::make_extended_thread, timing::tests::make_extended_timing,
        },
        misc::test_utils, msg_types::cred_issuance::CredentialIssuanceProtocolV1_0,
    };

    #[test]
    fn test_minimal_issue_cred() {
        let content = IssueCredentialContent::new(vec![make_extended_attachment()]);

        let decorators = IssueCredentialDecorators::new(make_extended_thread());

        let expected = json!({
            "credentials~attach": content.credentials_attach,
            "~thread": decorators.thread
        });

        test_utils::test_msg(content, decorators, CredentialIssuanceProtocolV1_0::IssueCredential, expected);
    }

    #[test]
    fn test_extended_issue_cred() {
        let mut content = IssueCredentialContent::new(vec![make_extended_attachment()]);
        content.comment = Some("test_comment".to_owned());

        let mut decorators = IssueCredentialDecorators::new(make_extended_thread());
        decorators.timing = Some(make_extended_timing());
        decorators.please_ack = Some(make_minimal_please_ack());

        let expected = json!({
            "credentials~attach": content.credentials_attach,
            "comment": content.comment,
            "~thread": decorators.thread,
            "~timing": decorators.timing,
            "~please_ack": decorators.please_ack
        });

        test_utils::test_msg(content, decorators, CredentialIssuanceProtocolV1_0::IssueCredential, expected);
    }
}
