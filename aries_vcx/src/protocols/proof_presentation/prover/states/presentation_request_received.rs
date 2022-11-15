use std::sync::Arc;

use crate::core::profile::profile::Profile;
use crate::error::prelude::*;
use crate::xyz::proofs::prover::prover::generate_indy_proof;
use messages::problem_report::ProblemReport;
use messages::proof_presentation::presentation::Presentation;
use messages::proof_presentation::presentation_request::PresentationRequest;
use messages::status::Status;
use crate::protocols::proof_presentation::prover::states::finished::FinishedState;
use crate::protocols::proof_presentation::prover::states::presentation_preparation_failed::PresentationPreparationFailedState;
use crate::protocols::proof_presentation::prover::states::presentation_prepared::PresentationPreparedState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PresentationRequestReceived {
    pub presentation_request: PresentationRequest,
}

impl PresentationRequestReceived {
    pub fn new(presentation_request: PresentationRequest) -> Self {
        Self { presentation_request }
    }

    pub async fn build_presentation(
        &self,
        profile: &Arc<dyn Profile>,
        credentials: &str,
        self_attested_attrs: &str,
    ) -> VcxResult<String> {
        
        generate_indy_proof(
            profile,
            credentials,
            self_attested_attrs,
            &self.presentation_request.request_presentations_attach.content()?,
        )
        .await
    }
}

impl From<(PresentationRequestReceived, ProblemReport)> for PresentationPreparationFailedState {
    fn from((state, problem_report): (PresentationRequestReceived, ProblemReport)) -> Self {
        trace!("transit state from PresentationRequestReceived to PresentationPreparationFailedState");
        PresentationPreparationFailedState {
            presentation_request: state.presentation_request,
            problem_report,
        }
    }
}

impl From<(PresentationRequestReceived, Presentation)> for PresentationPreparedState {
    fn from((state, presentation): (PresentationRequestReceived, Presentation)) -> Self {
        trace!("transit state from PresentationRequestReceived to PresentationPreparedState");
        PresentationPreparedState {
            presentation_request: state.presentation_request,
            presentation,
        }
    }
}

impl From<PresentationRequestReceived> for FinishedState {
    fn from(state: PresentationRequestReceived) -> Self {
        trace!("Prover: transit state from PresentationRequestReceived to FinishedState");
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: None,
            status: Status::Success,
        }
    }
}

impl From<(PresentationRequestReceived, ProblemReport)> for FinishedState {
    fn from((state, problem_report): (PresentationRequestReceived, ProblemReport)) -> Self {
        trace!("Prover: transit state from PresentationRequestReceived to FinishedState");
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: None,
            status: Status::Declined(problem_report),
        }
    }
}
