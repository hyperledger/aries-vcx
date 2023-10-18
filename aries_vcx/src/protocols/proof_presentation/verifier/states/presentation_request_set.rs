use messages::msg_fields::protocols::present_proof::v1::request::{
    RequestPresentationV1, RequestPresentationV1Content,
};
use uuid::Uuid;

use crate::protocols::proof_presentation::verifier::states::presentation_request_sent::PresentationRequestSentState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationRequestSetState {
    pub presentation_request: RequestPresentationV1,
}

impl Default for PresentationRequestSetState {
    fn default() -> Self {
        let id = Uuid::new_v4().to_string();
        let content = RequestPresentationV1Content::builder()
            .request_presentations_attach(Vec::new())
            .build();

        Self {
            presentation_request: RequestPresentationV1::builder()
                .id(id)
                .content(content)
                .build(),
        }
    }
}

impl PresentationRequestSetState {
    pub fn new(presentation_request: RequestPresentationV1) -> Self {
        Self {
            presentation_request,
        }
    }
}

impl From<PresentationRequestSetState> for PresentationRequestSentState {
    fn from(state: PresentationRequestSetState) -> Self {
        trace!("transit state from PresentationRequestSetState to PresentationRequestSentState");
        PresentationRequestSentState {
            presentation_request: state.presentation_request,
        }
    }
}
