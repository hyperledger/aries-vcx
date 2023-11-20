use messages::msg_fields::protocols::present_proof::v1::{
    propose::{PresentationPreview, ProposePresentationV1, ProposePresentationV1Content},
    request::RequestPresentationV1,
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationProposalReceivedState {
    pub presentation_proposal: ProposePresentationV1,
    pub presentation_request: Option<RequestPresentationV1>,
}

impl Default for PresentationProposalReceivedState {
    fn default() -> Self {
        let id = Uuid::new_v4().to_string();
        let preview = PresentationPreview::new(Vec::new(), Vec::new());

        let content = ProposePresentationV1Content::builder()
            .presentation_proposal(preview)
            .build();

        Self {
            presentation_proposal: ProposePresentationV1::builder()
                .id(id)
                .content(content)
                .build(),
            presentation_request: None,
        }
    }
}

impl PresentationProposalReceivedState {
    pub fn new(presentation_proposal: ProposePresentationV1) -> Self {
        Self {
            presentation_proposal,
            ..Self::default()
        }
    }
}
