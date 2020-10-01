use v3::handlers::proof_presentation::prover::states::finished::FinishedState;
use v3::messages::error::ProblemReport;
use v3::messages::proof_presentation::presentation::Presentation;
use v3::messages::proof_presentation::presentation_ack::PresentationAck;
use v3::messages::proof_presentation::presentation_request::PresentationRequest;
use v3::messages::status::Status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationSentState {
    pub connection_handle: u32,
    pub presentation_request: PresentationRequest,
    pub presentation: Presentation,
}

impl From<(PresentationSentState, PresentationAck)> for FinishedState {
    fn from((state, _ack): (PresentationSentState, PresentationAck)) -> Self {
        trace!("transit state from PresentationSentState to FinishedState");
        FinishedState {
            connection_handle: state.connection_handle,
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
            connection_handle: state.connection_handle,
            presentation_request: state.presentation_request,
            presentation: state.presentation,
            status: Status::Failed(problem_report),
        }
    }
}
