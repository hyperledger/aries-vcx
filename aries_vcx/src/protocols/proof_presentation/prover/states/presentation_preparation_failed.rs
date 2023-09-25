use messages::msg_fields::protocols::{
    present_proof::request::RequestPresentation, report_problem::ProblemReport,
};

use crate::{
    handlers::util::Status, protocols::proof_presentation::prover::states::finished::FinishedState,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationPreparationFailedState {
    pub presentation_request: RequestPresentation,
    pub problem_report: ProblemReport,
}

impl From<PresentationPreparationFailedState> for FinishedState {
    fn from(state: PresentationPreparationFailedState) -> Self {
        trace!("transit state from PresentationPreparationFailedState to FinishedState");
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: None,
            status: Status::Failed(state.problem_report),
        }
    }
}
