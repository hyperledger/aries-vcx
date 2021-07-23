use crate::handlers::connection::inviter::states::null::NullState;
use crate::handlers::connection::inviter::states::requested::RequestedState;
use crate::messages::connection::invite::Invitation;
use crate::messages::connection::problem_report::ProblemReport;
use crate::messages::connection::request::Request;
use crate::messages::connection::response::SignedResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitedState {
    pub invitation: Invitation,
}

impl From<(InvitedState, ProblemReport)> for NullState {
    fn from((_state, _error): (InvitedState, ProblemReport)) -> NullState {
        trace!("ConnectionInviter: transit state from InvitedState to NullState");
        NullState {}
    }
}

impl From<(InvitedState, Request, SignedResponse)> for RequestedState {
    fn from((_state, request, signed_response): (InvitedState, Request, SignedResponse)) -> RequestedState {
        trace!("ConnectionInviter: transit state from InvitedState to RespondedState");
        RequestedState {
            signed_response,
            did_doc: request.connection.did_doc,
            thread_id: request.id.0,
        }
    }
}
