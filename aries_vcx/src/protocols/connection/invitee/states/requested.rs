use crate::protocols::connection::invitee::states::initial::InitialState;
use crate::protocols::connection::invitee::states::responded::RespondedState;
use messages::diddoc::aries::diddoc::AriesDidDoc;
use messages::protocols::connection::problem_report::ProblemReport;
use messages::protocols::connection::request::Request;
use messages::protocols::connection::response::Response;

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
        InitialState::new(Some(problem_report), _state.did_doc)
    }
}

impl From<(RequestedState, Response)> for RespondedState {
    fn from((state, response): (RequestedState, Response)) -> RespondedState {
        trace!("ConnectionInvitee: transit state from RequestedState to RespondedState");
        RespondedState {
            response,
            did_doc: state.did_doc,
            request: state.request,
        }
    }
}
