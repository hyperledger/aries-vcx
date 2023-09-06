use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::CredentialPreview;
use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type ProposeCredential = MsgParts<ProposeCredentialContent, ProposeCredentialDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct ProposeCredentialContent {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub credential_proposal: CredentialPreview,
    pub schema_id: String,
    pub cred_def_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct ProposeCredentialDecorators {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
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
        decorators::{thread::tests::make_extended_thread, timing::tests::make_extended_timing},
        misc::test_utils,
        msg_fields::protocols::cred_issuance::CredentialAttr,
        msg_types::cred_issuance::CredentialIssuanceTypeV1_0,
    };

    #[test]
    fn test_minimal_propose_cred() {
        let attribute = CredentialAttr::builder()
            .name("test_attribute_name".to_owned())
            .value("test_attribute_value".to_owned())
            .build();
        let preview = CredentialPreview::builder().attributes(vec![attribute]).build();
        let content = ProposeCredentialContent::builder()
            .credential_proposal(preview)
            .schema_id("test_schema_id".to_owned())
            .cred_def_id("test_cred_def_id".to_owned())
            .build();

        let decorators = ProposeCredentialDecorators::default();

        let expected = json!({
            "credential_proposal": content.credential_proposal,
            "schema_id": content.schema_id,
            "cred_def_id": content.cred_def_id,
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
        let attribute = CredentialAttr::builder()
            .name("test_attribute_name".to_owned())
            .value("test_attribute_value".to_owned())
            .build();
        let preview = CredentialPreview::builder().attributes(vec![attribute]).build();
        let mut content = ProposeCredentialContent::builder()
            .credential_proposal(preview)
            .schema_id("test_schema_id".to_owned())
            .cred_def_id("test_cred_def_id".to_owned())
            .build();

        content.comment = Some("test_comment".to_owned());

        let mut decorators = ProposeCredentialDecorators::default();
        decorators.thread = Some(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let expected = json!({
            "credential_proposal": content.credential_proposal,
            "schema_id": content.schema_id,
            "cred_def_id": content.cred_def_id,
            "comment": content.comment,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV1_0::ProposeCredential,
            expected,
        );
    }
}
