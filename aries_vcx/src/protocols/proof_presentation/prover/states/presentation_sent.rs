use messages::msg_fields::protocols::{
    present_proof::{ack::AckPresentation, present::Presentation, request::RequestPresentation},
    report_problem::ProblemReport,
};

use crate::{
    handlers::util::Status, protocols::proof_presentation::prover::states::finished::FinishedState,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationSentState {
    pub presentation_request: RequestPresentation,
    pub presentation: Presentation,
}

impl From<(PresentationSentState, AckPresentation)> for FinishedState {
    fn from((state, _ack): (PresentationSentState, AckPresentation)) -> Self {
        trace!("transit state from PresentationSentState to FinishedState");
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: Some(state.presentation),
            status: Status::Success,
        }
    }
}

impl From<(PresentationSentState, ProblemReport)> for FinishedState {
    fn from((state, problem_report): (PresentationSentState, ProblemReport)) -> Self {
        trace!("transit state from PresentationSentState to FinishedState");
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: Some(state.presentation),
            status: Status::Failed(problem_report),
        }
    }
}
