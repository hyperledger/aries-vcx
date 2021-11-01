use crate::messages::proof_presentation::presentation_proposal::PresentationProposal;
use crate::messages::proof_presentation::presentation_request::PresentationRequestData;

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
