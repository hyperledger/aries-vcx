use messages::msg_fields::protocols::cred_issuance::ack::{AckCredential, AckCredentialContent};
use messages::msg_fields::protocols::cred_issuance::issue_credential::IssueCredential;
use messages::msg_fields::protocols::cred_issuance::offer_credential::OfferCredential;
use messages::msg_fields::protocols::cred_issuance::propose_credential::ProposeCredential;
use messages::msg_fields::protocols::cred_issuance::request_credential::RequestCredential;
use messages::msg_fields::protocols::cred_issuance::CredentialIssuance;
use messages::msg_fields::protocols::notification::Notification;
use messages::msg_fields::protocols::report_problem::ProblemReport;
use messages::msg_parts::MsgParts;
use messages::AriesMessage;

use crate::handlers::util::{matches_opt_thread_id, matches_thread_id};

type OptionalComment = Option<String>;

#[derive(Debug, Clone)]
pub enum CredentialIssuanceAction {
    CredentialSend(),
    CredentialProposalSend(ProposeCredential),
    CredentialProposal(ProposeCredential),
    CredentialOffer(OfferCredential),
    CredentialOfferReject(OptionalComment),
    CredentialRequestSend(String),
    CredentialRequest(RequestCredential),
    Credential(IssueCredential),
    CredentialAck(AckCredential),
    ProblemReport(ProblemReport),
    Unknown,
}

impl CredentialIssuanceAction {
    pub fn thread_id_matches(&self, thread_id: &str) -> bool {
        match self {
            Self::CredentialOffer(msg) => matches_opt_thread_id!(msg, thread_id),
            Self::CredentialProposal(msg) => matches_opt_thread_id!(msg, thread_id),
            Self::Credential(msg) => matches_thread_id!(msg, thread_id),
            _ => true, // doesn't seem right...
        }
    }
}

impl From<AriesMessage> for CredentialIssuanceAction {
    fn from(msg: AriesMessage) -> Self {
        match msg {
            AriesMessage::CredentialIssuance(CredentialIssuance::ProposeCredential(proposal)) => {
                CredentialIssuanceAction::CredentialProposal(proposal)
            }
            AriesMessage::CredentialIssuance(CredentialIssuance::OfferCredential(offer)) => {
                CredentialIssuanceAction::CredentialOffer(offer)
            }
            AriesMessage::CredentialIssuance(CredentialIssuance::RequestCredential(request)) => {
                CredentialIssuanceAction::CredentialRequest(request)
            }
            AriesMessage::CredentialIssuance(CredentialIssuance::IssueCredential(credential)) => {
                CredentialIssuanceAction::Credential(credential)
            }
            AriesMessage::CredentialIssuance(CredentialIssuance::Ack(ack)) => {
                CredentialIssuanceAction::CredentialAck(ack)
            }
            AriesMessage::Notification(Notification::Ack(ack)) => {
                let MsgParts {
                    id,
                    content,
                    decorators,
                } = ack;
                let ack = AckCredential::with_decorators(id, AckCredentialContent(content), decorators);
                CredentialIssuanceAction::CredentialAck(ack)
            }
            AriesMessage::ReportProblem(report) => CredentialIssuanceAction::ProblemReport(report),
            AriesMessage::Notification(Notification::ProblemReport(report)) => {
                let MsgParts {
                    id,
                    content,
                    decorators,
                } = report;
                let report = ProblemReport::with_decorators(id, content.0, decorators);
                CredentialIssuanceAction::ProblemReport(report)
            }
            _ => CredentialIssuanceAction::Unknown,
        }
    }
}
