use messages::a2a::A2AMessage;
use messages::issuance::revocation_notification::RevocationNotification;
use messages::problem_report::ProblemReport;
use messages::issuance::credential::Credential;
use messages::issuance::credential_ack::CredentialAck;
use messages::issuance::credential_offer::CredentialOffer;
use messages::issuance::credential_proposal::{CredentialProposal, CredentialProposalData};
use messages::issuance::credential_request::CredentialRequest;

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
    RevocationNotification(RevocationNotification),
    ProblemReport(ProblemReport),
    Unknown,
}

impl CredentialIssuanceAction {
    pub fn thread_id_matches(&self, thread_id: &str) -> bool {
        match self {
            Self::CredentialOffer(credential_offer) => credential_offer.from_thread(thread_id),
            Self::CredentialProposal(credential_proposal) => credential_proposal.from_thread(thread_id),
            Self::Credential(credential) => credential.from_thread(thread_id),
            Self::RevocationNotification(notification) => notification.from_thread(thread_id),
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
            A2AMessage::RevocationNotification(notification) => CredentialIssuanceAction::RevocationNotification(notification),
            _ => CredentialIssuanceAction::Unknown,
        }
    }
}
