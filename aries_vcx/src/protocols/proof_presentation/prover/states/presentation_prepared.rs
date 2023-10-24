use messages::msg_fields::protocols::present_proof::v1::{
    present::PresentationV1, request::RequestPresentationV1,
};

use crate::{
    handlers::util::Status,
    protocols::proof_presentation::prover::states::{
        finished::FinishedState, presentation_sent::PresentationSentState,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationPreparedState {
    pub presentation_request: RequestPresentationV1,
    pub presentation: PresentationV1,
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
