use crate::handlers::proof_presentation::verifier::state_machine::RevocationStatus;
use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::messages::status::Status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinishedState {
    pub presentation_request: PresentationRequest,
    pub presentation: Option<Presentation>,
    pub status: Status,
    pub revocation_status: Option<RevocationStatus>,
}
