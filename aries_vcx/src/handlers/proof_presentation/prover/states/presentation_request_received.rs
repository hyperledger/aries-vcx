use crate::error::prelude::*;
use crate::handlers::proof_presentation::prover::states::presentation_prepared::PresentationPreparedState;
use crate::handlers::proof_presentation::prover::states::presentation_prepared_failed::PresentationPreparationFailedState;
use crate::libindy::proofs::prover::prover::generate_indy_proof;
use crate::messages::error::ProblemReport;
use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::proof_presentation::presentation_request::PresentationRequest;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PresentationRequestReceived {
    pub presentation_request: PresentationRequest,
}


impl PresentationRequestReceived {
    pub fn new(presentation_request: PresentationRequest) -> Self {
        Self { presentation_request }
    }

    pub fn build_presentation(&self, credentials: &str, self_attested_attrs: &str) -> VcxResult<String> {
        generate_indy_proof(credentials,
                            self_attested_attrs,
                            &self.presentation_request.request_presentations_attach.content()?)
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
