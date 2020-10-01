use disclosed_proof::DisclosedProof;
use error::prelude::*;
use v3::handlers::proof_presentation::prover::states::presentation_prepared::PresentationPreparedState;
use v3::handlers::proof_presentation::prover::states::presentation_prepared_failed::PresentationPreparationFailedState;
use v3::messages::error::ProblemReport;
use v3::messages::proof_presentation::presentation::Presentation;
use v3::messages::proof_presentation::presentation_request::PresentationRequest;
use v3::messages::status::Status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitialState {
    pub presentation_request: PresentationRequest,
}


impl InitialState {
    pub fn build_presentation(&self, credentials: &str, self_attested_attrs: &str) -> VcxResult<String> {
        DisclosedProof::generate_indy_proof(credentials,
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