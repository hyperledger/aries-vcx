use crate::did_doc::DidDoc;
use crate::messages::connection::problem_report::ProblemReport;
use crate::messages::connection::request::Request;
use crate::messages::connection::response::SignedResponse;
use crate::protocols::connection::invitee::states::initial::InitialState;
use crate::protocols::connection::invitee::states::responded::RespondedState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestedState {
    pub request: Request,
    pub did_doc: DidDoc,
}

impl From<(RequestedState, ProblemReport)> for InitialState {
    fn from((_state, problem_report): (RequestedState, ProblemReport)) -> InitialState {
        trace!("ConnectionInvitee: transit state from RequestedState to InitialState, problem_report: {:?}", problem_report);
        InitialState::new(Some(problem_report))
    }
}

impl From<(RequestedState, SignedResponse)> for RespondedState {
    fn from((state, response): (RequestedState, SignedResponse)) -> RespondedState {
        trace!("ConnectionInvitee: transit state from RequestedState to RespondedState");
        RespondedState { response, did_doc: state.did_doc, request: state.request }
    }
}
