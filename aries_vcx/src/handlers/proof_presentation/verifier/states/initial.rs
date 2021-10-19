use crate::handlers::proof_presentation::verifier::states::presentation_request_sent::PresentationRequestSentState;
use crate::messages::proof_presentation::presentation_request::{PresentationRequest, PresentationRequestData};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct InitialState {
    pub presentation_request_data: PresentationRequestData,
}
