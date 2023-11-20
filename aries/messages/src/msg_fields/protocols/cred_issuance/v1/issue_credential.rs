use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{attachment::Attachment, please_ack::PleaseAck, thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type IssueCredentialV1 = MsgParts<IssueCredentialV1Content, IssueCredentialV1Decorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct IssueCredentialV1Content {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "credentials~attach")]
    pub credentials_attach: Vec<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct IssueCredentialV1Decorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{
            attachment::tests::make_extended_attachment,
            please_ack::tests::make_minimal_please_ack, thread::tests::make_extended_thread,
            timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_types::cred_issuance::CredentialIssuanceTypeV1_0,
    };

    #[test]
    fn test_minimal_issue_cred() {
        let content = IssueCredentialV1Content::builder()
            .credentials_attach(vec![make_extended_attachment()])
            .build();

        let decorators = IssueCredentialV1Decorators::builder()
            .thread(make_extended_thread())
            .build();

        let expected = json!({
            "credentials~attach": content.credentials_attach,
            "~thread": decorators.thread
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV1_0::IssueCredential,
            expected,
        );
    }

    #[test]
    fn test_extended_issue_cred() {
        let content = IssueCredentialV1Content::builder()
            .credentials_attach(vec![make_extended_attachment()])
            .comment("test_comment".to_owned())
            .build();

        let decorators = IssueCredentialV1Decorators::builder()
            .thread(make_extended_thread())
            .timing(make_extended_timing())
            .please_ack(make_minimal_please_ack())
            .build();

        let expected = json!({
            "credentials~attach": content.credentials_attach,
            "comment": content.comment,
            "~thread": decorators.thread,
            "~timing": decorators.timing,
            "~please_ack": decorators.please_ack
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV1_0::IssueCredential,
            expected,
        );
    }
}
