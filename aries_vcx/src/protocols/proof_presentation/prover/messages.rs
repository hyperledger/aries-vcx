use std::collections::HashMap;

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

use crate::handlers::{
    proof_presentation::types::SelectedCredentials,
    util::{matches_opt_thread_id, matches_thread_id, PresentationProposalData},
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum PresentationActions {
    PresentationProposalSend(PresentationProposalData),
    PresentationRequestReceived(RequestPresentation),
    RejectPresentationRequest(String),
    SetPresentation(Presentation),
    PreparePresentation((SelectedCredentials, HashMap<String, String>)),
    SendPresentation,
    PresentationAckReceived(AckPresentation),
    PresentationRejectReceived(ProblemReport),
    ProposePresentation(PresentationPreview),
    Unknown,
}

impl PresentationActions {
    pub fn thread_id_matches(&self, thread_id: &str) -> bool {
        match self {
            Self::SetPresentation(msg) => matches_thread_id!(msg, thread_id),
            Self::PresentationRejectReceived(msg) => matches_opt_thread_id!(msg, thread_id),
            Self::PresentationAckReceived(msg) => matches_thread_id!(msg, thread_id),
            _ => true,
        }
    }
}

impl From<AriesMessage> for PresentationActions {
    fn from(msg: AriesMessage) -> Self {
        match msg {
            AriesMessage::Notification(Notification::Ack(ack)) => {
                let MsgParts {
                    id,
                    content,
                    decorators,
                } = ack;
                let ack = AckPresentation::with_decorators(id, AckPresentationContent(content), decorators);
                PresentationActions::PresentationAckReceived(ack)
            }
            AriesMessage::PresentProof(PresentProof::Ack(ack)) => PresentationActions::PresentationAckReceived(ack),
            AriesMessage::PresentProof(PresentProof::RequestPresentation(request)) => {
                PresentationActions::PresentationRequestReceived(request)
            }
            AriesMessage::ReportProblem(report) => PresentationActions::PresentationRejectReceived(report),
            AriesMessage::Notification(Notification::ProblemReport(report)) => {
                let MsgParts {
                    id,
                    content,
                    decorators,
                } = report;
                let report = ProblemReport::with_decorators(id, content.0, decorators);
                PresentationActions::PresentationRejectReceived(report)
            }
            AriesMessage::PresentProof(PresentProof::ProblemReport(report)) => {
                let MsgParts {
                    id,
                    content,
                    decorators,
                } = report;
                let report = ProblemReport::with_decorators(id, content.0, decorators);
                PresentationActions::PresentationRejectReceived(report)
            }
            _ => PresentationActions::Unknown,
        }
    }
}
