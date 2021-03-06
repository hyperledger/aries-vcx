use crate::error::prelude::*;
use crate::aries::messages::a2a::{A2AMessage, MessageId};
use crate::aries::messages::attachment::{AttachmentId, Attachments};
use crate::aries::messages::connection::service::Service;
use crate::libindy::proofs::proof_request::ProofRequestData;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct PresentationRequest {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "request_presentations~attach")]
    pub request_presentations_attach: Attachments,
}

impl PresentationRequest {
    pub fn create() -> Self {
        PresentationRequest::default()
    }

    pub fn set_id(mut self, id: String) -> Self {
        self.id = MessageId(id);
        self
    }

    pub fn set_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn set_request_presentations_attach(mut self, request_presentations: &PresentationRequestData) -> VcxResult<PresentationRequest> {
        self.request_presentations_attach.add_base64_encoded_json_attachment(AttachmentId::PresentationRequest, json!(request_presentations))?;
        Ok(self)
    }

    pub fn to_json(&self) -> VcxResult<String> {
        serde_json::to_string(self)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize PresentationRequest: {}", err)))
    }
}

a2a_message!(PresentationRequest);


pub type PresentationRequestData = ProofRequestData;

#[cfg(test)]
pub mod tests {
    use crate::aries::messages::thread::Thread;
    use crate::aries::messages::connection::service::tests::_service;

    use super::*;

    pub fn _presentation_request_data() -> PresentationRequestData {
        PresentationRequestData::default()
            .set_requested_attributes(json!([{"name": "name"}]).to_string()).unwrap()
    }

    fn _attachment() -> Attachments {
        let mut attachment = Attachments::new();
        attachment.add_base64_encoded_json_attachment(AttachmentId::PresentationRequest, json!(_presentation_request_data())).unwrap();
        attachment
    }

    fn _comment() -> String {
        String::from("comment")
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
            comment: Some(_comment()),
            request_presentations_attach: _attachment(),
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_presentation_request_build_works() {
        let presentation_request: PresentationRequest = PresentationRequest::default()
            .set_comment(_comment())
            .set_request_presentations_attach(&_presentation_request_data()).unwrap();

        assert_eq!(_presentation_request(), presentation_request);
    }
}
