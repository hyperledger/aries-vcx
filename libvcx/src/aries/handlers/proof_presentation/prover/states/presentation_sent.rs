use crate::aries::handlers::proof_presentation::prover::states::finished::FinishedState;
use crate::aries::messages::error::ProblemReport;
use crate::aries::messages::proof_presentation::presentation::Presentation;
use crate::aries::messages::proof_presentation::presentation_ack::PresentationAck;
use crate::aries::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::aries::messages::status::Status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationSentState {
    pub presentation_request: PresentationRequest,
    pub presentation: Presentation,
}

impl From<(PresentationSentState, PresentationAck)> for FinishedState {
    fn from((state, _ack): (PresentationSentState, PresentationAck)) -> Self {
        trace!("transit state from PresentationSentState to FinishedState");
        FinishedState {
            presentation_request: state.presentation_request,
            presentation: state.presentation,
            status: Status::Success,
        }
    }
}

impl From<(PresentationSentState, ProblemReport)> for FinishedState {
    fn from((state, problem_report): (PresentationSentState, ProblemReport)) -> Self {
        trace!("transit state from PresentationSentState to FinishedState");
        FinishedState {
            presentation_request: state.presentation_request,
            presentation: state.presentation,
            status: Status::Failed(problem_report),
        }
    }
}
