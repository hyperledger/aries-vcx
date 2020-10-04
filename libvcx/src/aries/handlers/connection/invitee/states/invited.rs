use aries::handlers::connection::invitee::states::null::NullState;
use aries::handlers::connection::invitee::states::requested::RequestedState;
use aries::messages::connection::did_doc::DidDoc;
use aries::messages::connection::invite::Invitation;
use aries::messages::connection::problem_report::ProblemReport;
use aries::messages::connection::request::Request;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitedState {
    pub invitation: Invitation
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
