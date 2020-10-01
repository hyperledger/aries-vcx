use v3::messages::proof_presentation::presentation::Presentation;
use v3::messages::proof_presentation::presentation_request::PresentationRequest;
use v3::messages::status::Status;
use v3::handlers::proof_presentation::prover::states::initial::InitialState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinishedState {
    pub connection_handle: u32,
    pub presentation_request: PresentationRequest,
    pub presentation: Presentation,
    pub status: Status,
}


impl From<InitialState> for FinishedState {
    fn from(state: InitialState) -> Self {
        trace!("transit state from InitialState to FinishedState");
        FinishedState {
            connection_handle: 0,
            presentation_request: state.presentation_request,
            presentation: Default::default(),
            status: Status::Declined,
        }
    }
}


