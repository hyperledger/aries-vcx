use messages::msg_fields::protocols::present_proof::{
    propose::{PresentationPreview, ProposePresentation, ProposePresentationContent},
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

        let content = ProposePresentationContent::builder()
            .presentation_proposal(preview)
            .build();

        Self {
            presentation_proposal: ProposePresentation::builder().id(id).content(content).build(),
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
