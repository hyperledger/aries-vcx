use crate::messages::a2a::A2AMessage;
use crate::messages::ack::Ack;
use crate::messages::connection::invite::Invitation;
use crate::messages::connection::problem_report::ProblemReport;
use crate::messages::connection::request::Request;
use crate::messages::connection::response::SignedResponse;
use crate::messages::discovery::disclose::Disclose;
use crate::messages::discovery::query::Query;
use crate::messages::trust_ping::ping::Ping;
use crate::messages::trust_ping::ping_response::PingResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DidExchangeMessages {
    Connect(),
    InvitationReceived(Invitation),
    ExchangeRequestReceived(Request),
    ExchangeResponseReceived(SignedResponse),
    AckReceived(Ack),
    ProblemReportReceived(ProblemReport),
    SendPing(Option<String>),
    PingReceived(Ping),
    PingResponseReceived(PingResponse),
    DiscoverFeatures((Option<String>, Option<String>)),
    QueryReceived(Query),
    DiscloseReceived(Disclose),
    Unknown,
}

impl From<A2AMessage> for DidExchangeMessages {
    fn from(msg: A2AMessage) -> Self {
        match msg {
            A2AMessage::ConnectionInvitation(invite) => {
                DidExchangeMessages::InvitationReceived(invite)
            }
            A2AMessage::ConnectionRequest(request) => {
                DidExchangeMessages::ExchangeRequestReceived(request)
            }
            A2AMessage::ConnectionResponse(request) => {
                DidExchangeMessages::ExchangeResponseReceived(request)
            }
            A2AMessage::Ping(ping) => {
                DidExchangeMessages::PingReceived(ping)
            }
            A2AMessage::PingResponse(ping_response) => {
                DidExchangeMessages::PingResponseReceived(ping_response)
            }
            A2AMessage::Ack(ack) => {
                DidExchangeMessages::AckReceived(ack)
            }
            A2AMessage::Query(query) => {
                DidExchangeMessages::QueryReceived(query)
            }
            A2AMessage::Disclose(disclose) => {
                DidExchangeMessages::DiscloseReceived(disclose)
            }
            A2AMessage::ConnectionProblemReport(report) => {
                DidExchangeMessages::ProblemReportReceived(report)
            }
            _ => {
                DidExchangeMessages::Unknown
            }
        }
    }
}