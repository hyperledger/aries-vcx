use crate::handlers::proof_presentation::prover::states::initial::InitialState;
use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::messages::status::Status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinishedState {
    pub presentation_request: PresentationRequest,
    pub presentation: Presentation,
    pub status: Status,
}


impl From<InitialState> for FinishedState {
    fn from(state: InitialState) -> Self {
        trace!("transit state from InitialState to FinishedState");
        FinishedState {
            presentation_request: state.presentation_request,
            presentation: Default::default(),
            status: Status::Declined,
        }
    }
}


