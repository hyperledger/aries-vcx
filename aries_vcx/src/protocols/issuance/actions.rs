use messages::{
    a2a::A2AMessage,
    concepts::problem_report::ProblemReport,
    protocols::issuance::{
        credential::Credential,
        credential_ack::CredentialAck,
        credential_offer::CredentialOffer,
        credential_proposal::{CredentialProposal, CredentialProposalData},
        credential_request::CredentialRequest,
    },
};

type OptionalComment = Option<String>;

#[derive(Debug, Clone)]
pub enum CredentialIssuanceAction {
    CredentialSend(),
    CredentialProposalSend(CredentialProposalData),
    CredentialProposal(CredentialProposal),
    CredentialOffer(CredentialOffer),
    CredentialOfferReject(OptionalComment),
    CredentialRequestSend(String),
    CredentialRequest(CredentialRequest),
    Credential(Credential),
    CredentialAck(CredentialAck),
    ProblemReport(ProblemReport),
    Unknown,
}

impl CredentialIssuanceAction {
    pub fn thread_id_matches(&self, thread_id: &str) -> bool {
        match self {
            Self::CredentialOffer(credential_offer) => credential_offer.from_thread(thread_id),
            Self::CredentialProposal(credential_proposal) => credential_proposal.from_thread(thread_id),
            Self::Credential(credential) => credential.from_thread(thread_id),
            _ => true,
        }
    }
}

impl From<A2AMessage> for CredentialIssuanceAction {
    fn from(msg: A2AMessage) -> Self {
        match msg {
            A2AMessage::CredentialProposal(proposal) => CredentialIssuanceAction::CredentialProposal(proposal),
            A2AMessage::CredentialOffer(offer) => CredentialIssuanceAction::CredentialOffer(offer),
            A2AMessage::CredentialRequest(request) => CredentialIssuanceAction::CredentialRequest(request),
            A2AMessage::Credential(credential) => CredentialIssuanceAction::Credential(credential),
            A2AMessage::Ack(ack) | A2AMessage::CredentialAck(ack) => CredentialIssuanceAction::CredentialAck(ack),
            A2AMessage::CommonProblemReport(report) => CredentialIssuanceAction::ProblemReport(report),
            _ => CredentialIssuanceAction::Unknown,
        }
    }
}
