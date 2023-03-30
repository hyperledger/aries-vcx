use diddoc::aries::diddoc::AriesDidDoc;
use messages::msg_fields::protocols::connection::problem_report::ProblemReport;
use messages::msg_fields::protocols::connection::request::Request;
use messages::msg_fields::protocols::connection::ConnectionData;

use crate::protocols::mediated_connection::invitee::states::initial::InitialState;
use crate::protocols::mediated_connection::invitee::states::responded::RespondedState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestedState {
    pub request: Request,
    pub did_doc: AriesDidDoc,
}

impl From<(RequestedState, ProblemReport)> for InitialState {
    fn from((_state, problem_report): (RequestedState, ProblemReport)) -> InitialState {
        trace!(
            "ConnectionInvitee: transit state from RequestedState to InitialState, problem_report: {:?}",
            problem_report
        );
        InitialState::new(Some(problem_report), None)
    }
}

impl From<(RequestedState, ConnectionData)> for RespondedState {
    fn from((state, response): (RequestedState, ConnectionData)) -> RespondedState {
        trace!("ConnectionInvitee: transit state from RequestedState to RespondedState");
        RespondedState {
            resp_con_data: response,
            did_doc: state.did_doc,
            request: state.request,
        }
    }
}
