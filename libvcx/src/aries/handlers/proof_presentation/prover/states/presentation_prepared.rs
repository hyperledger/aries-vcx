use crate::aries::handlers::proof_presentation::prover::states::finished::FinishedState;
use crate::aries::handlers::proof_presentation::prover::states::presentation_sent::PresentationSentState;
use crate::aries::messages::proof_presentation::presentation::Presentation;
use crate::aries::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::aries::messages::status::Status;

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
            presentation_request: state.presentation_request,
            presentation: Default::default(),
            status: Status::Declined,
        }
    }
}
