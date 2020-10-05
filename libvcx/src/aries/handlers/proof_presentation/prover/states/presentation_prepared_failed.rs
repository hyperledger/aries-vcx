use aries::handlers::proof_presentation::prover::states::finished::FinishedState;
use aries::messages::error::ProblemReport;
use aries::messages::proof_presentation::presentation::Presentation;
use aries::messages::proof_presentation::presentation_request::PresentationRequest;
use aries::messages::status::Status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationPreparationFailedState {
    pub presentation_request: PresentationRequest,
    pub problem_report: ProblemReport,
}

impl From<(PresentationPreparationFailedState, u32)> for FinishedState {
    fn from((state, connection_handle): (PresentationPreparationFailedState, u32)) -> Self {
        trace!("transit state from PresentationPreparationFailedState to FinishedState");
        FinishedState {
            presentation_request: state.presentation_request,
            presentation: Presentation::create(),
            connection_handle,
            status: Status::Failed(state.problem_report),
        }
    }
}
