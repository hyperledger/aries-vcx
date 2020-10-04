use aries::handlers::proof_presentation::verifier::states::presentation_request_sent::PresentationRequestSentState;
use aries::messages::proof_presentation::presentation_request::{PresentationRequest, PresentationRequestData};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitialState {
    pub presentation_request_data: PresentationRequestData
}

impl From<(InitialState, PresentationRequest, u32)> for PresentationRequestSentState {
    fn from((_state, presentation_request, connection_handle): (InitialState, PresentationRequest, u32)) -> Self {
        trace!("transit state from InitialState to PresentationRequestSentState");
        PresentationRequestSentState { connection_handle, presentation_request }
    }
}
