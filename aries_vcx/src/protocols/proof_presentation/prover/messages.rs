use messages::{
    a2a::A2AMessage,
    concepts::problem_report::ProblemReport,
    protocols::proof_presentation::{
        presentation::Presentation,
        presentation_ack::PresentationAck,
        presentation_proposal::{PresentationPreview, PresentationProposalData},
        presentation_request::PresentationRequest,
    },
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ProverMessages {
    PresentationProposalSend(PresentationProposalData),
    PresentationRequestReceived(PresentationRequest),
    RejectPresentationRequest(String),
    SetPresentation(Presentation),
    PreparePresentation((String, String)),
    SendPresentation,
    PresentationAckReceived(PresentationAck),
    PresentationRejectReceived(ProblemReport),
    ProposePresentation(PresentationPreview),
    Unknown,
}

impl ProverMessages {
    pub fn thread_id_matches(&self, thread_id: &str) -> bool {
        match self {
            Self::SetPresentation(presentation) => presentation.from_thread(thread_id),
            Self::PresentationRejectReceived(problem_report) => problem_report.from_thread(thread_id),
            Self::PresentationAckReceived(ack) => ack.from_thread(thread_id),
            _ => true,
        }
    }
}

impl From<A2AMessage> for ProverMessages {
    fn from(msg: A2AMessage) -> Self {
        match msg {
            A2AMessage::Ack(ack) | A2AMessage::PresentationAck(ack) => ProverMessages::PresentationAckReceived(ack),
            A2AMessage::CommonProblemReport(report) => ProverMessages::PresentationRejectReceived(report),
            A2AMessage::PresentationRequest(request) => ProverMessages::PresentationRequestReceived(request),
            _ => ProverMessages::Unknown,
        }
    }
}
