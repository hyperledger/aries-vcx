use v3::handlers::proof_presentation::verifier::state_machine::RevocationStatus;
use v3::messages::proof_presentation::presentation::Presentation;
use v3::messages::proof_presentation::presentation_request::PresentationRequest;
use v3::messages::status::Status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinishedState {
    pub connection_handle: u32,
    pub presentation_request: PresentationRequest,
    pub presentation: Option<Presentation>,
    pub status: Status,
    pub revocation_status: Option<RevocationStatus>,
}