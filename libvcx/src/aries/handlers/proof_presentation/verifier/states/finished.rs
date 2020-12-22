use crate::aries::handlers::proof_presentation::verifier::state_machine::RevocationStatus;
use crate::aries::messages::proof_presentation::presentation::Presentation;
use crate::aries::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::aries::messages::status::Status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinishedState {
    pub presentation_request: PresentationRequest,
    pub presentation: Option<Presentation>,
    pub status: Status,
    pub revocation_status: Option<RevocationStatus>,
}