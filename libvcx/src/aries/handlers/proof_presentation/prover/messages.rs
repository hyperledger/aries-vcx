use crate::aries::messages::a2a::A2AMessage;
use crate::aries::messages::error::ProblemReport;
use crate::aries::messages::proof_presentation::presentation::Presentation;
use crate::aries::messages::proof_presentation::presentation_ack::PresentationAck;
use crate::aries::messages::proof_presentation::presentation_proposal::PresentationPreview;
use crate::aries::messages::proof_presentation::presentation_request::PresentationRequestData;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ProverMessages {
    PresentationRequestReceived(PresentationRequestData),
    RejectPresentationRequest(String),
    SetPresentation(Presentation),
    PreparePresentation((String, String)),
    SendPresentation,
    PresentationAckReceived(PresentationAck),
    PresentationRejectReceived(ProblemReport),
    ProposePresentation(PresentationPreview),
    Unknown,
}

impl From<A2AMessage> for ProverMessages {
    fn from(msg: A2AMessage) -> Self {
        match msg {
            A2AMessage::Ack(ack) | A2AMessage::PresentationAck(ack) => {
                ProverMessages::PresentationAckReceived(ack)
            }
            A2AMessage::CommonProblemReport(report) => {
                ProverMessages::PresentationRejectReceived(report)
            }
            _ => {
                ProverMessages::Unknown
            }
        }
    }
}