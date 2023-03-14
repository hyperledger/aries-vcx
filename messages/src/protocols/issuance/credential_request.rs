use crate::{
    a2a::{A2AMessage, MessageId},
    concepts::{
        attachment::{AttachmentId, Attachments},
        thread::Thread,
        timing::Timing,
    },
    errors::error::MessagesResult,
    timing_optional,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct CredentialRequest {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "requests~attach")]
    pub requests_attach: Attachments,
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

threadlike_optional!(CredentialRequest);
a2a_message!(CredentialRequest);
timing_optional!(CredentialRequest);

impl CredentialRequest {
    pub fn create() -> Self {
        CredentialRequest::default()
    }

    pub fn set_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn set_requests_attach(mut self, credential_request: String) -> MessagesResult<CredentialRequest> {
        self.requests_attach.add_base64_encoded_json_attachment(
            AttachmentId::CredentialRequest,
            serde_json::Value::String(credential_request),
        )?;
        Ok(self)
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use super::*;
    use crate::protocols::issuance::credential_offer::test_utils::{thread, thread_1};

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
        attachment
            .add_base64_encoded_json_attachment(AttachmentId::CredentialRequest, _attachment())
            .unwrap();

        CredentialRequest {
            id: MessageId::id(),
            comment: Some(_comment()),
            requests_attach: attachment,
            thread: Some(thread()),
            timing: None,
        }
    }

    pub fn _credential_request_1() -> CredentialRequest {
        let mut attachment = Attachments::new();
        attachment
            .add_base64_encoded_json_attachment(AttachmentId::CredentialRequest, _attachment())
            .unwrap();

        CredentialRequest {
            id: MessageId::id(),
            comment: Some(_comment()),
            requests_attach: attachment,
            thread: Some(thread_1()),
            timing: None,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;
    use crate::protocols::issuance::{credential_offer::test_utils::thread_id, credential_request::test_utils::*};

    #[test]
    fn test_credential_request_build_works() {
        let credential_request: CredentialRequest = CredentialRequest::create()
            .set_comment(_comment())
            .set_thread_id(&thread_id())
            .set_requests_attach(_attachment().to_string())
            .unwrap();

        assert_eq!(_credential_request(), credential_request);
    }
}
