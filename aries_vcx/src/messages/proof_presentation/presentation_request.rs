use crate::error::prelude::*;
use crate::messages::proof_presentation::presentation_request_data::ProofRequestData;
use crate::messages::a2a::{A2AMessage, MessageId};
use crate::messages::attachment::{AttachmentId, Attachments};
use crate::messages::thread::Thread;
use crate::messages::timing::Timing;
use crate::timing_optional;

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
        request_presentations: &PresentationRequestData,
    ) -> VcxResult<PresentationRequest> {
        trace!("set_request_presentations_attach >>> {:?}", request_presentations);
        self.request_presentations_attach
            .add_base64_encoded_json_attachment(AttachmentId::PresentationRequest, json!(request_presentations))?;
        Ok(self)
    }

    pub fn get_presentation_request_data(self) -> VcxResult<ProofRequestData> {
        let content = &self.request_presentations_attach.content()?;
        serde_json::from_str(content).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::InvalidJson,
                format!(
                    "Cannot deserialize PresentationRequestData: {}, error: {}",
                    content, err
                ),
            )
        })
    }

    pub fn to_json(&self) -> VcxResult<String> {
        serde_json::to_string(self).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::InvalidJson,
                format!("Cannot serialize PresentationRequest: {}", err),
            )
        })
    }
}

pub type PresentationRequestData = ProofRequestData;

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use crate::messages::thread::Thread;

    use super::*;

    pub fn _presentation_request_data() -> PresentationRequestData {
        PresentationRequestData::default()
            .set_requested_attributes_as_string(json!([{"name": "name"}]).to_string())
            .unwrap()
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
    use crate::messages::proof_presentation::presentation_request::test_utils::*;
    use crate::utils::devsetup::was_in_past;
    

    use super::*;

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
