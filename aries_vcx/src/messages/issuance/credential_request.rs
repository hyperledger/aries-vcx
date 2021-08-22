use crate::error::VcxResult;
use crate::messages::a2a::{A2AMessage, MessageId};
use crate::messages::attachment::{AttachmentId, Attachments};
use crate::messages::thread::Thread;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct CredentialRequest {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "requests~attach")]
    pub requests_attach: Attachments,
    #[serde(rename = "~thread")]
    pub thread: Thread,
}

impl CredentialRequest {
    pub fn create() -> Self {
        CredentialRequest::default()
    }

    pub fn set_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn set_requests_attach(mut self, credential_request: String) -> VcxResult<CredentialRequest> {
        self.requests_attach.add_base64_encoded_json_attachment(AttachmentId::CredentialRequest, serde_json::Value::String(credential_request))?;
        Ok(self)
    }
}

threadlike!(CredentialRequest);
a2a_message!(CredentialRequest);

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use crate::messages::issuance::credential_offer::test_utils::thread;

    use super::*;

    pub fn _attachment() -> serde_json::Value {
        json!({
            "prover_did":"VsKV7grR1BUE29mG2Fm2kX",
            "cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG1"
        })
    }

    pub fn _comment() -> String {
        String::from("comment")
    }

    pub fn _my_pw_did() -> String {
        String::from("VsKV7grR1BUE29mG2Fm2kX")
    }

    pub fn _credential_request() -> CredentialRequest {
        let mut attachment = Attachments::new();
        attachment.add_base64_encoded_json_attachment(AttachmentId::CredentialRequest, _attachment()).unwrap();

        CredentialRequest {
            id: MessageId::id(),
            comment: Some(_comment()),
            requests_attach: attachment,
            thread: thread(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::messages::issuance::credential_offer::test_utils::thread_id;
    use crate::messages::issuance::credential_request::test_utils::*;

    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_credential_request_build_works() {
        let credential_request: CredentialRequest = CredentialRequest::create()
            .set_comment(_comment())
            .set_thread_id(&thread_id())
            .set_requests_attach(_attachment().to_string()).unwrap();

        assert_eq!(_credential_request(), credential_request);
    }
}
