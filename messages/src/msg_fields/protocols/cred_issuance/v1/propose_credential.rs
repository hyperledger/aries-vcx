use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::CredentialPreviewV1;
use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type ProposeCredentialV1 = MsgParts<ProposeCredentialV1Content, ProposeCredentialV1Decorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct ProposeCredentialV1Content {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub credential_proposal: CredentialPreviewV1,
    pub schema_id: String,
    pub cred_def_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct ProposeCredentialV1Decorators {
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
        msg_fields::protocols::cred_issuance::v1::CredentialAttr,
        msg_types::cred_issuance::CredentialIssuanceTypeV1_0,
    };

    #[test]
    fn test_minimal_propose_cred() {
        let attribute = CredentialAttr::builder()
            .name("test_attribute_name".to_owned())
            .value("test_attribute_value".to_owned())
            .build();
        let preview = CredentialPreviewV1::new(vec![attribute]);
        let content = ProposeCredentialV1Content::builder()
            .credential_proposal(preview)
            .schema_id("test_schema_id".to_owned())
            .cred_def_id("test_cred_def_id".to_owned())
            .build();

        let decorators = ProposeCredentialV1Decorators::default();

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
        let preview = CredentialPreviewV1::new(vec![attribute]);
        let content = ProposeCredentialV1Content::builder()
            .credential_proposal(preview)
            .schema_id("test_schema_id".to_owned())
            .cred_def_id("test_cred_def_id".to_owned())
            .comment("test_comment".to_owned())
            .build();

        let decorators = ProposeCredentialV1Decorators::builder()
            .thread(make_extended_thread())
            .timing(make_extended_timing())
            .build();

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
