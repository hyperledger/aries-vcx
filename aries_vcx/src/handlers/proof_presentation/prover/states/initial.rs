use crate::error::prelude::*;
use crate::handlers::proof_presentation::prover::states::presentation_prepared::PresentationPreparedState;
use crate::handlers::proof_presentation::prover::states::presentation_prepared_failed::PresentationPreparationFailedState;
use crate::messages::error::ProblemReport;
use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::libindy::proofs::prover::prover::generate_indy_proof;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct InitialState {
    pub presentation_request: PresentationRequest,
}


impl InitialState {
    pub fn build_presentation(&self, credentials: &str, self_attested_attrs: &str) -> VcxResult<String> {
        generate_indy_proof(credentials,
                                            self_attested_attrs,
                                            &self.presentation_request.request_presentations_attach.content()?)
    }
}

impl From<(InitialState, ProblemReport)> for PresentationPreparationFailedState {
    fn from((state, problem_report): (InitialState, ProblemReport)) -> Self {
        trace!("transit state from InitialState to PresentationPreparationFailedState");
        PresentationPreparationFailedState {
            presentation_request: state.presentation_request,
            problem_report,
        }
    }
}

impl From<(InitialState, Presentation)> for PresentationPreparedState {
    fn from((state, presentation): (InitialState, Presentation)) -> Self {
        trace!("transit state from InitialState to PresentationPreparedState");
        PresentationPreparedState {
            presentation_request: state.presentation_request,
            presentation,
        }
    }
}
