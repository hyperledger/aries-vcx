use crate::messages::a2a::A2AMessage;
use crate::messages::error::ProblemReport;
use crate::messages::issuance::credential::Credential;
use crate::messages::issuance::credential_ack::CredentialAck;
use crate::messages::issuance::credential_offer::CredentialOffer;
use crate::messages::issuance::credential_proposal::CredentialProposal;
use crate::messages::issuance::credential_request::CredentialRequest;

#[derive(Debug, Clone)]
pub enum CredentialIssuanceMessage {
    CredentialInit(Option<String>),
    CredentialSend(),
    CredentialProposalSend(CredentialProposal),
    CredentialProposal(CredentialProposal),
    CredentialOffer(CredentialOffer),
    CredentialRequestSend(String),
    CredentialRequest(CredentialRequest),
    Credential(Credential),
    CredentialAck(CredentialAck),
    ProblemReport(ProblemReport),
    Unknown,
}

impl CredentialIssuanceMessage {
    pub fn thread_id_matches(&self, thread_id: &str) -> bool {
        match self {
            Self::CredentialOffer(credential_offer) => credential_offer.from_thread(thread_id),
            Self::CredentialProposal(credential_proposal) => credential_proposal.from_thread(thread_id),
            Self::Credential(credential) => credential.from_thread(thread_id),
            _ => true
        }
    }
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
