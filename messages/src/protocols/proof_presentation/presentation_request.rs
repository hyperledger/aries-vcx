use crate::{
    a2a::{A2AMessage, MessageId},
    concepts::{
        attachment::{AttachmentId, Attachments},
        thread::Thread,
        timing::Timing,
    },
    errors::error::prelude::*,
    timing_optional,
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct PresentationRequest {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "request_presentations~attach")]
    pub request_presentations_attach: Attachments,
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

timing_optional!(PresentationRequest);
threadlike_optional!(PresentationRequest);
a2a_message!(PresentationRequest);

impl PresentationRequest {
    pub fn create() -> Self {
        PresentationRequest::default()
    }

    pub fn set_id(mut self, id: String) -> Self {
        self.id = MessageId(id);
        self
    }

    pub fn set_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    pub fn set_request_presentations_attach(
        mut self,
        request_presentations: &str,
    ) -> MessagesResult<PresentationRequest> {
        self.request_presentations_attach
            .add_base64_encoded_json_attachment(AttachmentId::PresentationRequest, json!(request_presentations))?;
        Ok(self)
    }

    pub fn get_presentation_request_data(self) -> MessagesResult<String> {
        self.request_presentations_attach.content()
    }

    pub fn to_json(&self) -> MessagesResult<String> {
        serde_json::to_string(self).map_err(|err| {
            MessagesError::from_msg(
                MessagesErrorKind::InvalidJson,
                format!("Cannot serialize PresentationRequest: {}", err),
            )
        })
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use super::*;
    use crate::concepts::thread::Thread;

    pub fn _presentation_request_data() -> String {
        json!({
            "nonce": "",
            "name": "",
            "version": "1.0",
            "requested_attributes": {
              "attribute_0": {
                "name": "",
                "restrictions": []
              },
            },
            "requested_predicates": {},
            "non_revoked": null,
            "ver": null
        })
        .to_string()
    }

    fn _attachment() -> Attachments {
        let mut attachment = Attachments::new();
        attachment
            .add_base64_encoded_json_attachment(AttachmentId::PresentationRequest, json!(_presentation_request_data()))
            .unwrap();
        attachment
    }

    pub fn _comment() -> Option<String> {
        Some(String::from("comment"))
    }

    pub fn thread_id() -> String {
        _presentation_request().id.0
    }

    pub fn thread() -> Thread {
        Thread::new().set_thid(_presentation_request().id.0)
    }

    pub fn _presentation_request() -> PresentationRequest {
        PresentationRequest {
            id: MessageId::id(),
            comment: _comment(),
            request_presentations_attach: _attachment(),
            thread: None,
            timing: Some(Timing::default()),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;
    use crate::{protocols::proof_presentation::presentation_request::test_utils::*, utils::devsetup::was_in_past};

    #[test]
    fn test_presentation_request_build_works() {
        let presentation_request: PresentationRequest = PresentationRequest::create()
            .set_comment(_comment())
            .set_request_presentations_attach(&_presentation_request_data())
            .unwrap()
            .set_out_time();

        let expected = _presentation_request();
        assert_eq!(expected.id, presentation_request.id);
        assert_eq!(expected.comment, presentation_request.comment);
        assert_eq!(
            expected.request_presentations_attach,
            presentation_request.request_presentations_attach
        );
        assert_eq!(expected.thread, presentation_request.thread);
        assert!(presentation_request.timing.is_some());
        let out_timestamp: String = presentation_request.timing.unwrap().get_out_time().unwrap().into();
        assert!(was_in_past(&out_timestamp, chrono::Duration::milliseconds(100)).unwrap());
    }
}
