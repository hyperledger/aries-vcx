use messages::{
    msg_fields::protocols::{
        notification::Notification,
        present_proof::{
            ack::{AckPresentation, AckPresentationContent},
            present::Presentation,
            propose::PresentationPreview,
            request::RequestPresentation,
            PresentProof,
        },
        report_problem::ProblemReport,
    },
    msg_parts::MsgParts,
    AriesMessage,
};

use crate::handlers::util::{matches_opt_thread_id, matches_thread_id, PresentationProposalData};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ProverMessages {
    PresentationProposalSend(PresentationProposalData),
    PresentationRequestReceived(RequestPresentation),
    RejectPresentationRequest(String),
    SetPresentation(Presentation),
    PreparePresentation((String, String)),
    SendPresentation,
    PresentationAckReceived(AckPresentation),
    PresentationRejectReceived(ProblemReport),
    ProposePresentation(PresentationPreview),
    Unknown,
}

impl ProverMessages {
    pub fn thread_id_matches(&self, thread_id: &str) -> bool {
        match self {
            Self::SetPresentation(msg) => matches_thread_id!(msg, thread_id),
            Self::PresentationRejectReceived(msg) => matches_opt_thread_id!(msg, thread_id),
            Self::PresentationAckReceived(msg) => matches_thread_id!(msg, thread_id),
            _ => true,
        }
    }
}

impl From<AriesMessage> for ProverMessages {
    fn from(msg: AriesMessage) -> Self {
        match msg {
            AriesMessage::Notification(Notification::Ack(ack)) => {
                let MsgParts {
                    id,
                    content,
                    decorators,
                } = ack;
                let ack = AckPresentation::with_decorators(id, AckPresentationContent(content), decorators);
                ProverMessages::PresentationAckReceived(ack)
            }
            AriesMessage::PresentProof(PresentProof::Ack(ack)) => ProverMessages::PresentationAckReceived(ack),
            AriesMessage::ReportProblem(report) => ProverMessages::PresentationRejectReceived(report),
            AriesMessage::PresentProof(PresentProof::RequestPresentation(request)) => {
                ProverMessages::PresentationRequestReceived(request)
            }
            _ => ProverMessages::Unknown,
        }
    }
}
