use messages::msg_fields::protocols::present_proof::v1::propose::{
    PresentationPreview, ProposePresentationV1, ProposePresentationV1Content,
};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PresentationProposalSent {
    pub proposal: ProposePresentationV1,
}

impl Default for PresentationProposalSent {
    fn default() -> Self {
        let id = Uuid::new_v4().to_string();
        let preview = PresentationPreview::new(Vec::new(), Vec::new());

        let content = ProposePresentationV1Content::builder()
            .presentation_proposal(preview)
            .build();

        Self {
            proposal: ProposePresentationV1::builder()
                .id(id)
                .content(content)
                .build(),
        }
    }
}

impl PresentationProposalSent {
    pub fn new(proposal: ProposePresentationV1) -> Self {
        Self { proposal }
    }
}
