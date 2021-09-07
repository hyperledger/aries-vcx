use crate::handlers::connection::invitee::states::null::NullState;
use crate::handlers::connection::invitee::states::requested::RequestedState;
use crate::messages::connection::did_doc::DidDoc;
use crate::messages::connection::invite::Invitation;
use crate::messages::connection::problem_report::ProblemReport;
use crate::messages::connection::request::Request;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvitedState {
    pub invitation: Invitation,
}

impl From<(InvitedState, ProblemReport)> for NullState {
    fn from((_state, _error): (InvitedState, ProblemReport)) -> NullState {
        trace!("ConnectionInvitee: transit state from InvitedState to NullState");
        NullState {}
    }
}

impl From<(InvitedState, Request)> for RequestedState {
    fn from((state, request): (InvitedState, Request)) -> RequestedState {
        trace!("ConnectionInvitee: transit state from InvitedState to RequestedState");
        RequestedState { request, did_doc: DidDoc::from(state.invitation) }
    }
}
