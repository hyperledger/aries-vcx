use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::messages::status::Status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinishedState {
    pub presentation_request: PresentationRequest, // TODO: Should be option to avoid using default
    pub presentation: Presentation, // TODO: Should be option to avoid using default
    pub status: Status,
}

impl FinishedState {
    pub fn declined() -> Self {
        trace!("transit state to FinishedState due to a rejection");
        FinishedState {
            presentation_request: Default::default(),
            presentation: Default::default(),
            status: Status::Declined,
        }
    }
}
