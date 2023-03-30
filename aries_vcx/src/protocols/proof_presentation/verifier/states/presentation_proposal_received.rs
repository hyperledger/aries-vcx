use messages2::msg_fields::protocols::present_proof::{
    propose::{PresentationPreview, ProposePresentation, ProposePresentationContent, ProposePresentationDecorators},
    request::RequestPresentation,
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationProposalReceivedState {
    pub presentation_proposal: ProposePresentation,
    pub presentation_request: Option<RequestPresentation>,
}

impl Default for PresentationProposalReceivedState {
    fn default() -> Self {
        let id = Uuid::new_v4().to_string();
        let preview = PresentationPreview::new(Vec::new(), Vec::new());
        let content = ProposePresentationContent::new(preview);
        let decorators = ProposePresentationDecorators::default();

        Self {
            presentation_proposal: ProposePresentation::with_decorators(id, content, decorators),
            presentation_request: None,
        }
    }
}

impl PresentationProposalReceivedState {
    pub fn new(presentation_proposal: ProposePresentation) -> Self {
        Self {
            presentation_proposal,
            ..Self::default()
        }
    }
}
