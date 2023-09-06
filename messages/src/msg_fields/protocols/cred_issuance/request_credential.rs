use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{attachment::Attachment, thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type RequestCredential = MsgParts<RequestCredentialContent, RequestCredentialDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct RequestCredentialContent {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "requests~attach")]
    pub requests_attach: Vec<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct RequestCredentialDecorators {
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{attachment::tests::make_extended_attachment, thread::tests::make_extended_thread},
        misc::test_utils,
        msg_types::cred_issuance::CredentialIssuanceTypeV1_0,
    };

    #[test]
    fn test_minimal_request_cred() {
        let content = RequestCredentialContent::builder()
            .requests_attach(vec![make_extended_attachment()])
            .build();

        let decorators = RequestCredentialDecorators::default();

        let expected = json!({
            "requests~attach": content.requests_attach,
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV1_0::RequestCredential,
            expected,
        );
    }

    #[test]
    fn test_extended_request_cred() {
        let mut content = RequestCredentialContent::builder()
            .requests_attach(vec![make_extended_attachment()])
            .build();
        content.comment = Some("test_comment".to_owned());

        let mut decorators = RequestCredentialDecorators::default();
        decorators.thread = Some(make_extended_thread());

        let expected = json!({
            "requests~attach": content.requests_attach,
            "comment": content.comment,
            "~thread": decorators.thread
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV1_0::RequestCredential,
            expected,
        );
    }
}
