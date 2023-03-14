use messages::protocols::proof_presentation::{
    presentation_proposal::PresentationProposal, presentation_request::PresentationRequest,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PresentationProposalReceivedState {
    pub presentation_proposal: PresentationProposal,
    pub presentation_request: Option<PresentationRequest>,
}

impl PresentationProposalReceivedState {
    pub fn new(presentation_proposal: PresentationProposal) -> Self {
        Self {
            presentation_proposal,
            ..Self::default()
        }
    }
}
