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
