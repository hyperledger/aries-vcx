use messages::{
    a2a::A2AMessage,
    concepts::problem_report::ProblemReport,
    protocols::proof_presentation::{
        presentation::Presentation, presentation_proposal::PresentationProposal,
        presentation_request::PresentationRequest,
    },
};

type Reason = String;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum VerifierMessages {
    VerifyPresentation(Presentation),
    RejectPresentationProposal(Reason),
    SetPresentationRequest(PresentationRequest),
    PresentationProposalReceived(PresentationProposal),
    PresentationRejectReceived(ProblemReport),
    SendPresentationAck(),
    Unknown,
}

impl VerifierMessages {
    pub fn thread_id_matches(&self, thread_id: &str) -> bool {
        match self {
            Self::VerifyPresentation(presentation) => presentation.from_thread(thread_id),
            Self::PresentationProposalReceived(proposal) => proposal.from_thread(thread_id),
            Self::PresentationRejectReceived(problem_report) => problem_report.from_thread(thread_id),
            _ => true,
        }
    }
}

impl From<A2AMessage> for VerifierMessages {
    fn from(msg: A2AMessage) -> Self {
        match msg {
            A2AMessage::Presentation(presentation) => VerifierMessages::VerifyPresentation(presentation),
            A2AMessage::PresentationProposal(presentation_proposal) => {
                VerifierMessages::PresentationProposalReceived(presentation_proposal)
            }
            A2AMessage::CommonProblemReport(report) => VerifierMessages::PresentationRejectReceived(report),
            _ => VerifierMessages::Unknown,
        }
    }
}
