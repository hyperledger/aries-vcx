use aries::handlers::proof_presentation::verifier::state_machine::RevocationStatus;
use aries::messages::proof_presentation::presentation::Presentation;
use aries::messages::proof_presentation::presentation_request::PresentationRequest;
use aries::messages::status::Status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinishedState {
    pub connection_handle: u32,
    pub presentation_request: PresentationRequest,
    pub presentation: Option<Presentation>,
    pub status: Status,
    pub revocation_status: Option<RevocationStatus>,
}