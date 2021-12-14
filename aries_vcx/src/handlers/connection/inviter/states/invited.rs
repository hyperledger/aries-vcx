use crate::handlers::connection::inviter::states::initial::InitialState;
use crate::handlers::connection::inviter::states::requested::RequestedState;
use crate::messages::connection::invite::Invitation;
use crate::messages::connection::problem_report::ProblemReport;
use crate::messages::connection::request::Request;
use crate::messages::connection::response::SignedResponse;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvitedState {
    pub invitation: Invitation,
}

// TODO: These have no justification for being here anymore
impl From<ProblemReport> for InitialState {
    fn from(problem_report: ProblemReport) -> InitialState {
        trace!("ConnectionInviter: transit state to InitialState");
        InitialState::new(Some(problem_report))
    }
}

impl From<(Request, SignedResponse)> for RequestedState {
    fn from((request, signed_response): (Request, SignedResponse)) -> RequestedState {
        trace!("ConnectionInviter: transit state to RespondedState");
        RequestedState {
            signed_response,
            did_doc: request.connection.did_doc,
            thread_id: request.id.0,
        }
    }
}
