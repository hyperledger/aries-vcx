use crate::aries::handlers::proof_presentation::prover::states::finished::FinishedState;
use crate::aries::messages::error::ProblemReport;
use crate::aries::messages::proof_presentation::presentation::Presentation;
use crate::aries::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::aries::messages::status::Status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationPreparationFailedState {
    pub presentation_request: PresentationRequest,
    pub problem_report: ProblemReport,
}

impl From<(PresentationPreparationFailedState)> for FinishedState {
    fn from((state): (PresentationPreparationFailedState)) -> Self {
        trace!("transit state from PresentationPreparationFailedState to FinishedState");
        FinishedState {
            presentation_request: state.presentation_request,
            presentation: Presentation::create(),
            status: Status::Failed(state.problem_report),
        }
    }
}
