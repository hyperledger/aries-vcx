use crate::messages::proof_presentation::presentation_proposal::PresentationProposal;
use crate::messages::proof_presentation::presentation_request::{PresentationRequest, PresentationRequestData};
use crate::handlers::proof_presentation::verifier::states::presentation_request_sent::PresentationRequestSentState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PresentationProposalReceivedState {
    pub presentation_proposal: PresentationProposal,
    pub presentation_request_data: Option<PresentationRequestData>,
}

impl PresentationProposalReceivedState {
    pub fn new(presentation_proposal: PresentationProposal) -> Self {
        Self {
            presentation_proposal,
            ..Self::default()
        }
    }
}
