use messages::{
    protocols::proof_presentation::{presentation::Presentation, presentation_request::PresentationRequest},
    status::Status,
};

use crate::protocols::proof_presentation::prover::states::{
    finished::FinishedState, presentation_sent::PresentationSentState,
};

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
