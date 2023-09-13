use messages::msg_fields::protocols::present_proof::request::{RequestPresentation, RequestPresentationContent};
use uuid::Uuid;

use crate::protocols::proof_presentation::verifier::states::presentation_request_sent::PresentationRequestSentState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationRequestSetState {
    pub presentation_request: RequestPresentation,
}

impl Default for PresentationRequestSetState {
    fn default() -> Self {
        let id = Uuid::new_v4().to_string();
        let content = RequestPresentationContent::builder()
            .request_presentations_attach(Vec::new())
            .build();

        Self {
            presentation_request: RequestPresentation::builder().id(id).content(content).build(),
        }
    }
}

impl PresentationRequestSetState {
    pub fn new(presentation_request: RequestPresentation) -> Self {
        Self { presentation_request }
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
