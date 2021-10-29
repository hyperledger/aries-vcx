use crate::handlers::proof_presentation::prover::states::finished::FinishedState;
use crate::messages::error::ProblemReport;
use crate::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::messages::status::Status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationPreparationFailedState {
    pub presentation_request: PresentationRequest,
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
