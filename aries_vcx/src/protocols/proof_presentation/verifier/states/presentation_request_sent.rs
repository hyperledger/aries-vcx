use std::sync::Arc;

use crate::common::proofs::verifier::verifier::validate_indy_proof;
use crate::core::profile::profile::Profile;
use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};
use crate::global::settings;
use crate::protocols::proof_presentation::verifier::states::finished::FinishedState;
use crate::protocols::proof_presentation::verifier::verification_status::PresentationVerificationStatus;
use messages::concepts::problem_report::ProblemReport;
use messages::protocols::proof_presentation::presentation::Presentation;
use messages::protocols::proof_presentation::presentation_request::PresentationRequest;
use messages::status::Status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationRequestSentState {
    pub presentation_request: PresentationRequest,
}

impl PresentationRequestSentState {
    pub async fn verify_presentation(
        &self,
        profile: &Arc<dyn Profile>,
        presentation: &Presentation,
        thread_id: &str,
    ) -> VcxResult<()> {
        if !settings::indy_mocks_enabled() && !presentation.from_thread(thread_id) {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!(
                    "Cannot handle proof presentation: thread id does not match: {:?}",
                    presentation.thread
                ),
            ));
        };

        let valid = validate_indy_proof(
            profile,
            &presentation.presentations_attach.content()?,
            &self.presentation_request.request_presentations_attach.content()?,
        )
        .await?;

        if !valid {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidProof,
                "Presentation verification failed",
            ));
        }

        Ok(())
    }
}

impl
    From<(
        PresentationRequestSentState,
        Presentation,
        PresentationVerificationStatus,
    )> for FinishedState
{
    fn from(
        (state, presentation, verification_status): (
            PresentationRequestSentState,
            Presentation,
            PresentationVerificationStatus,
        ),
    ) -> Self {
        trace!("transit state from PresentationRequestSentState to FinishedState");
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: Some(presentation),
            status: Status::Success,
            verification_status: verification_status,
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
            verification_status: PresentationVerificationStatus::Unavailable(),
        }
    }
}
