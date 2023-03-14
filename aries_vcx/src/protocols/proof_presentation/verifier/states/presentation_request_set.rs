use messages::protocols::proof_presentation::presentation_request::PresentationRequest;

use crate::protocols::proof_presentation::verifier::states::presentation_request_sent::PresentationRequestSentState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PresentationRequestSetState {
    pub presentation_request: PresentationRequest,
}

impl PresentationRequestSetState {
    pub fn new(presentation_request: PresentationRequest) -> Self {
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
