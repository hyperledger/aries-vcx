use aries::messages::a2a::A2AMessage;
use aries::messages::error::ProblemReport;
use aries::messages::issuance::credential::Credential;
use aries::messages::issuance::credential_ack::CredentialAck;
use aries::messages::issuance::credential_offer::CredentialOffer;
use aries::messages::issuance::credential_proposal::CredentialProposal;
use aries::messages::issuance::credential_request::CredentialRequest;

#[derive(Debug, Clone)]
pub enum CredentialIssuanceMessage {
    CredentialInit(u32),
    CredentialSend(u32),
    CredentialProposal(CredentialProposal),
    CredentialOffer(CredentialOffer),
    CredentialRequestSend(u32),
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
