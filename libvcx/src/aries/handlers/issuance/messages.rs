use crate::aries::messages::a2a::A2AMessage;
use crate::aries::messages::error::ProblemReport;
use crate::aries::messages::issuance::credential::Credential;
use crate::aries::messages::issuance::credential_ack::CredentialAck;
use crate::aries::messages::issuance::credential_offer::CredentialOffer;
use crate::aries::messages::issuance::credential_proposal::CredentialProposal;
use crate::aries::messages::issuance::credential_request::CredentialRequest;

#[derive(Debug, Clone)]
pub enum CredentialIssuanceMessage {
    CredentialInit(Option<String>),
    CredentialSend(),
    CredentialProposal(CredentialProposal),
    CredentialOffer(CredentialOffer),
    CredentialRequestSend(),
    CredentialRequest(CredentialRequest),
    Credential(Credential),
    CredentialAck(CredentialAck),
    ProblemReport(ProblemReport),
    Unknown,
}

impl From<A2AMessage> for CredentialIssuanceMessage {
    fn from(msg: A2AMessage) -> Self {
        match msg {
            A2AMessage::CredentialProposal(proposal) => {
                CredentialIssuanceMessage::CredentialProposal(proposal)
            }
            A2AMessage::CredentialOffer(offer) => {
                CredentialIssuanceMessage::CredentialOffer(offer)
            }
            A2AMessage::CredentialRequest(request) => {
                CredentialIssuanceMessage::CredentialRequest(request)
            }
            A2AMessage::Credential(credential) => {
                CredentialIssuanceMessage::Credential(credential)
            }
            A2AMessage::Ack(ack) | A2AMessage::CredentialAck(ack) => {
                CredentialIssuanceMessage::CredentialAck(ack)
            }
            A2AMessage::CommonProblemReport(report) => {
                CredentialIssuanceMessage::ProblemReport(report)
            }
            _ => {
                CredentialIssuanceMessage::Unknown
            }
        }
    }
}
