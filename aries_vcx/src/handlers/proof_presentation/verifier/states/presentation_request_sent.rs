use crate::error::{VcxError, VcxErrorKind, VcxResult};
use crate::handlers::proof_presentation::verifier::state_machine::RevocationStatus;
use crate::handlers::proof_presentation::verifier::states::finished::FinishedState;
use crate::libindy::proofs::verifier::verifier::validate_indy_proof;
use crate::messages::a2a::A2AMessage;
use crate::messages::error::ProblemReport;
use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::proof_presentation::presentation_ack::PresentationAck;
use crate::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::messages::status::Status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationRequestSentState {
    pub presentation_request: PresentationRequest,
}

impl PresentationRequestSentState {
    pub fn verify_presentation(&self, presentation: &Presentation, send_message: Option<&impl Fn(&A2AMessage) -> VcxResult<()>>) -> VcxResult<()> {
        if let Some(thread_id) = &self.presentation_request.thread.thid {
            if !presentation.from_thread(&thread_id) {
                return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot handle presentation: thread id does not match: {:?}", presentation.thread)));
            };
        };

        let valid = validate_indy_proof(&presentation.presentations_attach.content()?,
                                        &self.presentation_request.request_presentations_attach.content()?)?;

        if !valid {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidProof, "Presentation verification failed"));
        }

        if presentation.please_ack.is_some() {
            let ack = PresentationAck::create().set_thread_id(&self.presentation_request.id.0);
            send_message.ok_or(
                VcxError::from_msg(VcxErrorKind::InvalidState, "Attempted to call undefined send_message callback")
            )?(&A2AMessage::PresentationAck(ack))?;
        }

        Ok(())
    }
}


impl From<(PresentationRequestSentState, Presentation, RevocationStatus)> for FinishedState {
    fn from((state, presentation, was_revoked): (PresentationRequestSentState, Presentation, RevocationStatus)) -> Self {
        trace!("transit state from PresentationRequestSentState to FinishedState");
        FinishedState {
            presentation_request: state.presentation_request,
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
            presentation_request: state.presentation_request,
            presentation: None,
            status: Status::Failed(problem_report),
            revocation_status: None,
        }
    }
}
