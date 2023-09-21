use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::{AttachmentFormatSpecifier, CredentialPreviewV2};
use crate::{
    decorators::{attachment::Attachment, thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type ProposeCredentialV2 = MsgParts<ProposeCredentialV2Content, ProposeCredentialV2Decorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct ProposeCredentialV2Content {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<String>, // TODO - spec does not specify what goal codes to use..
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_preview: Option<CredentialPreviewV2>,
    pub formats: Vec<AttachmentFormatSpecifier<ProposeCredentialAttachmentFormatType>>,
    #[serde(rename = "filters~attach")]
    pub filters_attach: Vec<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct ProposeCredentialV2Decorators {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum ProposeCredentialAttachmentFormatType {
    #[serde(rename = "dif/credential-manifest@v1.0")]
    DifCredentialManifest1_0,
    #[serde(rename = "aries/ld-proof-vc-detail@v1.0")]
    AriesLdProofVcDetail1_0,
    #[serde(rename = "hlindy/cred-filter@v2.0")]
    HyperledgerIndyCredentialFilter2_0,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;
    use shared_vcx::maybe_known::MaybeKnown;

    use super::*;
    use crate::{
        decorators::{thread::tests::make_extended_thread, timing::tests::make_extended_timing},
        misc::test_utils,
        msg_fields::protocols::cred_issuance::CredentialAttr,
        msg_types::cred_issuance::CredentialIssuanceTypeV1_0,
    };

    #[test]
    fn propose_attachment_type_serialization() {
        let t: MaybeKnown<ProposeCredentialAttachmentFormatType, String> =
            MaybeKnown::Known(ProposeCredentialAttachmentFormatType::AriesLdProofVcDetail1_0);

        let s = serde_json::to_string(&t).unwrap();

        println!("{s}");

        let t: MaybeKnown<ProposeCredentialAttachmentFormatType> = serde_json::from_str(&s).unwrap();

        println!("{t:?}")
    }

    #[test]
    fn propose_attachment_type_serializatio2n() {
        let t: MaybeKnown<ProposeCredentialAttachmentFormatType> = MaybeKnown::Unknown(String::from("hello"));

        let s = serde_json::to_string(&t).unwrap();

        println!("{s}");

        let t: MaybeKnown<ProposeCredentialAttachmentFormatType> = serde_json::from_str(&s).unwrap();

        println!("{t:?}")
    }

    #[test]
    fn testeeeeee() {
        let val = json!([{
            "attach_id": "1",
            "format": "dif/credential-manifest@v1.0"
        },
        {
            "attach_id": "2",
            "format": "aries/ld-proof-vc-detail@v1.0"
        },
        {
            "attach_id": "1",
            "format": "hlindy/cred-filter@v2.0"
        },
        {
            "attach_id": "1",
            "format": "anoncreds/new-filters@v1.0"
        },
        ]);

        let resolved: Vec<AttachmentFormatSpecifier<ProposeCredentialAttachmentFormatType>> =
            serde_json::from_value(val).unwrap();

        println!("{resolved:?}");

        print!("{}", serde_json::to_string(&resolved).unwrap())
    }

    // #[test]
    // fn test_minimal_propose_cred() {
    //     let attribute = CredentialAttr::builder()
    //         .name("test_attribute_name".to_owned())
    //         .value("test_attribute_value".to_owned())
    //         .build();
    //     let preview = CredentialPreview::new(vec![attribute]);
    //     let content = ProposeCredentialV2Content::builder()
    //         .credential_proposal(preview)
    //         .schema_id("test_schema_id".to_owned())
    //         .cred_def_id("test_cred_def_id".to_owned())
    //         .build();

    //     let decorators = ProposeCredentialDecorators::default();

    //     let expected = json!({
    //         "credential_proposal": content.credential_proposal,
    //         "schema_id": content.schema_id,
    //         "cred_def_id": content.cred_def_id,
    //     });

    //     test_utils::test_msg(
    //         content,
    //         decorators,
    //         CredentialIssuanceTypeV1_0::ProposeCredential,
    //         expected,
    //     );
    // }

    // #[test]
    // fn test_extended_propose_cred() {
    //     let attribute = CredentialAttr::builder()
    //         .name("test_attribute_name".to_owned())
    //         .value("test_attribute_value".to_owned())
    //         .build();
    //     let preview = CredentialPreview::new(vec![attribute]);
    //     let content = ProposeCredentialV2Content::builder()
    //         .credential_proposal(preview)
    //         .comment("test_comment".to_owned())
    //         .format(vec![])
    //         .build();

    //     let decorators = ProposeCredentialV2Decorators::builder()
    //         .thread(make_extended_thread())
    //         .timing(make_extended_timing())
    //         .build();

    //     let expected = json!({
    //         "credential_proposal": content.credential_proposal,
    //         "schema_id": content.schema_id,
    //         "cred_def_id": content.cred_def_id,
    //         "comment": content.comment,
    //         "~thread": decorators.thread,
    //         "~timing": decorators.timing
    //     });

    //     test_utils::test_msg(
    //         content,
    //         decorators,
    //         CredentialIssuanceTypeV1_0::ProposeCredential,
    //         expected,
    //     );
    // }
}
