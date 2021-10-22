use crate::handlers::proof_presentation::prover::states::finished::FinishedState;
use crate::handlers::proof_presentation::prover::states::presentation_sent::PresentationSentState;
use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::messages::status::Status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationPreparedState {
    pub presentation_request: PresentationRequest,
    pub presentation: Presentation,
}

impl From<PresentationPreparedState> for PresentationSentState {
    fn from(state: PresentationPreparedState) -> Self {
        trace!("transit state from PresentationPreparedState to PresentationSentState");
        PresentationSentState {
            presentation_request: state.presentation_request,
            presentation: state.presentation,
        }
    }
}

impl From<PresentationPreparedState> for FinishedState {
    fn from(state: PresentationPreparedState) -> Self {
        trace!("transit state from PresentationPreparedState to FinishedState");
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: Default::default(),
            status: Status::Undefined,
        }
    }
}
