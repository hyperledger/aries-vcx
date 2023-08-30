use messages::msg_fields::protocols::present_proof::propose::{
    PresentationPreview, ProposePresentation, ProposePresentationContent, ProposePresentationDecorators,
};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PresentationProposalSet {
    pub proposal: ProposePresentation,
}

impl Default for PresentationProposalSet {
    fn default() -> Self {
        let id = Uuid::new_v4().to_string();
        let preview = PresentationPreview::new(Vec::new(), Vec::new());
        let content = ProposePresentationContent::new(preview);
        let decorators = ProposePresentationDecorators::default();

        Self {
            proposal: ProposePresentation::with_decorators(id, content, decorators),
        }
    }
}

impl PresentationProposalSet {
    pub fn new(proposal: ProposePresentation) -> Self {
        Self { proposal }
    }
}
