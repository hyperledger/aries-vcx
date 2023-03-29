use messages2::{
    msg_fields::protocols::{
        present_proof::{
            present::Presentation, propose::ProposePresentation, request::RequestPresentation, PresentProof,
        },
        report_problem::ProblemReport,
    },
    AriesMessage,
};

use crate::handlers::util::{matches_opt_thread_id, matches_thread_id};

type Reason = String;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum VerifierMessages {
    VerifyPresentation(Presentation),
    RejectPresentationProposal(Reason),
    SetPresentationRequest(RequestPresentation),
    PresentationProposalReceived(ProposePresentation),
    PresentationRejectReceived(ProblemReport),
    SendPresentationAck(),
    Unknown,
}

impl VerifierMessages {
    pub fn thread_id_matches(&self, thread_id: &str) -> bool {
        match self {
            Self::VerifyPresentation(msg) => matches_thread_id!(msg, thread_id),
            Self::PresentationProposalReceived(msg) => matches_thread_id!(msg, thread_id),
            Self::PresentationRejectReceived(msg) => matches_opt_thread_id!(msg, thread_id),
            _ => true,
        }
    }
}

impl From<AriesMessage> for VerifierMessages {
    fn from(msg: AriesMessage) -> Self {
        match msg {
            AriesMessage::PresentProof(PresentProof::Presentation(presentation)) => {
                VerifierMessages::VerifyPresentation(presentation)
            }
            AriesMessage::PresentProof(PresentProof::ProposePresentation(presentation_proposal)) => {
                VerifierMessages::PresentationProposalReceived(presentation_proposal)
            }
            AriesMessage::ReportProblem(report) => VerifierMessages::PresentationRejectReceived(report),
            _ => VerifierMessages::Unknown,
        }
    }
}
