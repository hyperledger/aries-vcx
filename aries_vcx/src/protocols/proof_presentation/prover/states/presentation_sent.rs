use messages::error::ProblemReport;
use messages::proof_presentation::presentation::Presentation;
use messages::proof_presentation::presentation_ack::PresentationAck;
use messages::proof_presentation::presentation_request::PresentationRequest;
use messages::status::Status;
use crate::protocols::proof_presentation::prover::states::finished::FinishedState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationSentState {
    pub presentation_request: PresentationRequest,
    pub presentation: Presentation,
}

impl From<(PresentationSentState, PresentationAck)> for FinishedState {
    fn from((state, _ack): (PresentationSentState, PresentationAck)) -> Self {
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
