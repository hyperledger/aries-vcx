use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{attachment::Attachment, please_ack::PleaseAck, thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

use super::AttachmentFormatSpecifier;

pub type IssueCredentialV2 = MsgParts<IssueCredentialV2Content, IssueCredentialV2Decorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct IssueCredentialV2Content {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement_id: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub formats: Vec<AttachmentFormatSpecifier<IssueCredentialAttachmentFormatType>>,
    #[serde(rename = "credentials~attach")]
    pub credentials_attach: Vec<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct IssueCredentialV2Decorators {
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum IssueCredentialAttachmentFormatType {
    #[serde(rename = "aries/ld-proof-vc@v1.0")]
    AriesLdProofVcDetail1_0,
    #[serde(rename = "hlindy/cred@v2.0")]
    HyperledgerIndyCredential2_0,
}

// #[cfg(test)]
// #[allow(clippy::unwrap_used)]
// #[allow(clippy::field_reassign_with_default)]
// mod tests {
//     use serde_json::json;

//     use super::*;
//     use crate::{
//         decorators::{
//             attachment::tests::make_extended_attachment, please_ack::tests::make_minimal_please_ack,
//             thread::tests::make_extended_thread, timing::tests::make_extended_timing,
//         },
//         misc::test_utils,
//         msg_types::cred_issuance::CredentialIssuanceTypeV1_0,
//     };

//     #[test]
//     fn test_minimal_issue_cred() {
//         let content = IssueCredentialContent::builder()
//             .credentials_attach(vec![make_extended_attachment()])
//             .build();

//         let decorators = IssueCredentialDecorators::builder()
//             .thread(make_extended_thread())
//             .build();

//         let expected = json!({
//             "credentials~attach": content.credentials_attach,
//             "~thread": decorators.thread
//         });

//         test_utils::test_msg(
//             content,
//             decorators,
//             CredentialIssuanceTypeV1_0::IssueCredential,
//             expected,
//         );
//     }

//     #[test]
//     fn test_extended_issue_cred() {
//         let content = IssueCredentialContent::builder()
//             .credentials_attach(vec![make_extended_attachment()])
//             .comment("test_comment".to_owned())
//             .build();

//         let decorators = IssueCredentialDecorators::builder()
//             .thread(make_extended_thread())
//             .timing(make_extended_timing())
//             .please_ack(make_minimal_please_ack())
//             .build();

//         let expected = json!({
//             "credentials~attach": content.credentials_attach,
//             "comment": content.comment,
//             "~thread": decorators.thread,
//             "~timing": decorators.timing,
//             "~please_ack": decorators.please_ack
//         });

//         test_utils::test_msg(
//             content,
//             decorators,
//             CredentialIssuanceTypeV1_0::IssueCredential,
//             expected,
//         );
//     }
// }
