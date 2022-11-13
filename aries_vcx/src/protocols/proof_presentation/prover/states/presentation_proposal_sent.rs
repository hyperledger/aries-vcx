use messages::proof_presentation::presentation_proposal::PresentationProposal;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PresentationProposalSent {
    proposal: PresentationProposal,
}

impl PresentationProposalSent {
    pub fn new(proposal: PresentationProposal) -> Self {
        Self { proposal }
    }
}
