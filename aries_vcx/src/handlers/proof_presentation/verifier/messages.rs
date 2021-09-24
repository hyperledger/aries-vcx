use crate::messages::a2a::A2AMessage;
use crate::messages::error::ProblemReport;
use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::proof_presentation::presentation_proposal::PresentationProposal;

type Comment = Option<String>;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum VerifierMessages {
    SendPresentationRequest(Comment),
    VerifyPresentation(Presentation),
    PresentationProposalReceived(PresentationProposal),
    PresentationRejectReceived(ProblemReport),
    Unknown,
}

impl VerifierMessages {
    pub fn thread_id_matches(&self, thread_id: &str) -> bool {
        match self {
            Self::VerifyPresentation(presentation) => presentation.from_thread(thread_id),
            Self::PresentationProposalReceived(proposal) => proposal.from_thread(thread_id),
            Self::PresentationRejectReceived(problem_report) => problem_report.from_thread(thread_id),
            _ => true
        }
    }
}

impl From<A2AMessage> for VerifierMessages {
    fn from(msg: A2AMessage) -> Self {
        match msg {
            A2AMessage::Presentation(presentation) => {
                VerifierMessages::VerifyPresentation(presentation)
            }
            A2AMessage::PresentationProposal(presentation_proposal) => {
                VerifierMessages::PresentationProposalReceived(presentation_proposal)
            }
            A2AMessage::CommonProblemReport(report) => {
                VerifierMessages::PresentationRejectReceived(report)
            }
            _ => {
                VerifierMessages::Unknown
            }
        }
    }
}
