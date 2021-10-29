use crate::handlers::connection::invitee::states::initial::InitialState;
use crate::handlers::connection::invitee::states::responded::RespondedState;
use crate::messages::connection::did_doc::DidDoc;
use crate::messages::connection::problem_report::ProblemReport;
use crate::messages::connection::request::Request;
use crate::messages::connection::response::SignedResponse;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestedState {
    pub request: Request,
    pub did_doc: DidDoc,
}

impl From<(RequestedState, ProblemReport)> for InitialState {
    fn from((_state, _error): (RequestedState, ProblemReport)) -> InitialState {
        trace!("ConnectionInvitee: transit state from RequestedState to InitialState");
        InitialState {}
    }
}

impl From<(RequestedState, SignedResponse)> for RespondedState {
    fn from((state, response): (RequestedState, SignedResponse)) -> RespondedState {
        trace!("ConnectionInvitee: transit state from RequestedState to RespondedState");
        RespondedState { response, did_doc: state.did_doc, request: state.request }
    }
}
