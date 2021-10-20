use crate::handlers::proof_presentation::prover::states::presentation_request_received::PresentationRequestReceived;
use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::messages::status::Status;
use crate::messages::error::ProblemReport;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinishedState {
    pub presentation_request: PresentationRequest, // TODO: Should be option to avoid using default
    pub presentation: Presentation, // TODO: Should be option to avoid using default
    pub status: Status,
}


impl From<PresentationRequestReceived> for FinishedState {
    fn from(state: PresentationRequestReceived) -> Self {
        trace!("Prover: transit state from PresentationRequestReceived to FinishedState");
        FinishedState {
            presentation_request: state.presentation_request,
            presentation: Default::default(),
            status: Status::Declined,
        }
    }
}

impl From<ProblemReport> for FinishedState {
    fn from(_: ProblemReport) -> Self {
        trace!("Prover: transit state from to FinishedState due to a problem");
        FinishedState {
            presentation_request: Default::default(),
            presentation: Default::default(),
            status: Status::Declined,
        }
    }
}
