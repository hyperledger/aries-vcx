use serde::{Deserialize, Serialize};

use super::CredentialPreview;
use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type ProposeCredential = MsgParts<ProposeCredentialContent, ProposeCredentialDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ProposeCredentialContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub credential_proposal: CredentialPreview,
    pub schema_id: String,
    pub cred_def_id: String,
}

impl ProposeCredentialContent {
    pub fn new(credential_proposal: CredentialPreview, schema_id: String, cred_def_id: String) -> Self {
        Self {
            comment: None,
            credential_proposal,
            schema_id,
            cred_def_id,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct ProposeCredentialDecorators {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
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
        decorators::{thread::tests::make_extended_thread, timing::tests::make_extended_timing},
        misc::test_utils,
        msg_fields::protocols::cred_issuance::CredentialAttr,
        msg_types::cred_issuance::CredentialIssuanceTypeV1_0,
    };

    #[test]
    fn test_minimal_propose_cred() {
        let attribute = CredentialAttr::new("test_attribute_name".to_owned(), "test_attribute_value".to_owned());
        let preview = CredentialPreview::new(vec![attribute]);
        let content =
            ProposeCredentialContent::new(preview, "test_schema_id".to_owned(), "test_cred_def_id".to_owned());

        let decorators = ProposeCredentialDecorators::default();

        let expected = json!({
            "credential_proposal": {
                "field1": "value1",
                "field2": "value2",
            },
            "schema_id": "result",
            "cred_def_id": "result"
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV1_0::ProposeCredential,
            expected,
        );
    }

    #[test]
    fn test_extended_propose_cred() {
        let attribute = CredentialAttr::new("test_attribute_name".to_owned(), "test_attribute_value".to_owned());
        let preview = CredentialPreview::new(vec![attribute]);
        let mut content =
            ProposeCredentialContent::new(preview, "test_schema_id".to_owned(), "test_cred_def_id".to_owned());

        content.comment = Some("test_comment".to_owned());

        let mut decorators = ProposeCredentialDecorators::default();
        decorators.thread = Some(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "credential_proposal": {
                "field1": "value1",
                "field2": "value2",
            },
            "schema_id": "result",
            "cred_def_id": "result",
            "comment": {
                "field1": "value1",
                "field2": "value2",
            },
            "~thread": {
                "field_1": "value1",
                "field_2": "value2",
            },
            "~timing": {
                "field1": "value1",
                "field2": "value2",
            }
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV1_0::ProposeCredential,
            expected,
        );
    }
}
