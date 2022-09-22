use messages::error::ProblemReport;
use messages::proof_presentation::presentation::Presentation;
use messages::proof_presentation::presentation_request::PresentationRequest;
use messages::status::Status;
use crate::protocols::proof_presentation::verifier::state_machine::RevocationStatus;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinishedState {
    pub presentation_request: Option<PresentationRequest>,
    pub presentation: Option<Presentation>,
    pub status: Status,
    pub revocation_status: Option<RevocationStatus>,
}

impl FinishedState {
    pub fn declined(problem_report: ProblemReport) -> Self {
        trace!("transit state to FinishedState due to a rejection");
        FinishedState {
            presentation_request: None,
            presentation: None,
            status: Status::Declined(problem_report),
            revocation_status: None,
        }
    }
}
