use error::VcxResult;
use error::prelude::*;
use messages::thread::Thread;
use aries::messages::a2a::{A2AMessage, MessageId};
use aries::messages::ack::PleaseAck;
use aries::messages::attachment::{AttachmentId, Attachments};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct Credential {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "credentials~attach")]
    pub credentials_attach: Attachments,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct CredentialData {
    pub schema_id: String,
    pub cred_def_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev_reg_id: Option<String>,
    pub values: serde_json::Value,
    pub signature: serde_json::Value,
    pub signature_correctness_proof: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev_reg: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness: Option<serde_json::Value>,
}

impl Credential {
    pub fn create() -> Self {
        Credential::default()
    }

    pub fn set_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn set_credential(mut self, credential: String) -> VcxResult<Credential> {
        self.credentials_attach.add_base64_encoded_json_attachment(AttachmentId::Credential, ::serde_json::Value::String(credential))?;
        Ok(self)
    }
}

please_ack!(Credential);
threadlike!(Credential);
a2a_message!(Credential);

#[cfg(test)]
pub mod tests {
    use aries::messages::issuance::credential_offer::tests::{thread, thread_id};

    use super::*;

    fn _attachment() -> ::serde_json::Value {
        json!({
            "schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0",
            "cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG1",
            "values":{"name":{"raw":"Name","encoded":"1139481716457488690172217916278103335"}}
        })
    }

    fn _comment() -> String {
        String::from("comment")
    }

    pub fn _credential() -> Credential {
        let mut attachment = Attachments::new();
        attachment.add_base64_encoded_json_attachment(AttachmentId::Credential, _attachment()).unwrap();

        Credential {
            id: MessageId::id(),
            comment: Some(_comment()),
            thread: thread(),
            credentials_attach: attachment,
            please_ack: None,
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_credential_build_works() {
        let credential: Credential = Credential::create()
            .set_comment(_comment())
            .set_thread_id(&thread_id())
            .set_credential(_attachment().to_string()).unwrap();

        assert_eq!(_credential(), credential);
    }
}
