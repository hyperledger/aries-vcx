use crate::protocols::connection::inviter::states::initial::InitialState;
use crate::protocols::connection::inviter::states::requested::RequestedState;
use messages::protocols::connection::invite::Invitation;
use messages::protocols::connection::problem_report::ProblemReport;
use messages::protocols::connection::request::Request;
use messages::protocols::connection::response::SignedResponse;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvitedState {
    pub invitation: Invitation,
}

// TODO: These have no justification for being here anymore
impl From<ProblemReport> for InitialState {
    fn from(problem_report: ProblemReport) -> InitialState {
        trace!(
            "ConnectionInviter: transit state to InitialState, problem_report: {:?}",
            problem_report
        );
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
