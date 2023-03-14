use std::sync::Arc;

use messages::{
    concepts::problem_report::ProblemReport,
    protocols::proof_presentation::{presentation::Presentation, presentation_request::PresentationRequest},
    status::Status,
};

use crate::{
    common::proofs::prover::prover::generate_indy_proof,
    core::profile::profile::Profile,
    errors::error::prelude::*,
    protocols::proof_presentation::prover::states::{
        finished::FinishedState, presentation_preparation_failed::PresentationPreparationFailedState,
        presentation_prepared::PresentationPreparedState,
    },
};

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
