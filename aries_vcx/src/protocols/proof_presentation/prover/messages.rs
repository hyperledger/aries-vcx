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
    RejectPresentationRequest(String),
    PreparePresentation((SelectedCredentials, HashMap<String, String>)),
    SendPresentation,
    ReceivePresentationAck(AckPresentation),
    ProposePresentation(PresentationPreview),
    Unknown,
}

impl PresentationActions {
    pub fn thread_id_matches(&self, thread_id: &str) -> bool {
        match self {
            Self::ReceivePresentationAck(msg) => matches_thread_id!(msg, thread_id),
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
                PresentationActions::ReceivePresentationAck(ack)
            }
            AriesMessage::PresentProof(PresentProof::Ack(ack)) => PresentationActions::ReceivePresentationAck(ack),
            _ => PresentationActions::Unknown,
        }
    }
}
