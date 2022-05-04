use crate::error::{VcxError, VcxErrorKind, VcxResult};
use crate::libindy::proofs::verifier::verifier::validate_indy_proof;
use crate::messages::error::ProblemReport;
use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::messages::status::Status;
use crate::protocols::proof_presentation::verifier::state_machine::RevocationStatus;
use crate::protocols::proof_presentation::verifier::states::finished::FinishedState;
use crate::settings;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationRequestSentState {
    pub presentation_request: PresentationRequest,
}

impl PresentationRequestSentState {
    pub async fn verify_presentation(&self, presentation: &Presentation, thread_id: &str) -> VcxResult<()> {
        if !settings::indy_mocks_enabled() && !presentation.from_thread(&thread_id) {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot handle proof presentation: thread id does not match: {:?}", presentation.thread)));
        };

        let valid = validate_indy_proof(&presentation.presentations_attach.content()?,
                                        &self.presentation_request.request_presentations_attach.content()?).await?;

        if !valid {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidProof, "Presentation verification failed"));
        }

        Ok(())
    }
}


impl From<(PresentationRequestSentState, Presentation, RevocationStatus)> for FinishedState {
    fn from((state, presentation, was_revoked): (PresentationRequestSentState, Presentation, RevocationStatus)) -> Self {
        trace!("transit state from PresentationRequestSentState to FinishedState");
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: Some(presentation),
            status: Status::Success,
            revocation_status: Some(was_revoked),
        }
    }
}

impl From<(PresentationRequestSentState, ProblemReport)> for FinishedState {
    fn from((state, problem_report): (PresentationRequestSentState, ProblemReport)) -> Self {
        trace!("transit state from PresentationRequestSentState to FinishedState");
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: None,
            status: Status::Failed(problem_report),
            revocation_status: None,
        }
    }
}
