use messages::msg_fields::protocols::connection::{
    problem_report::ProblemReport, request::Request, response::Response,
};

use crate::{
    handlers::util::AnyInvitation,
    protocols::mediated_connection::inviter::states::{
        initial::InitialState, requested::RequestedState,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvitedState {
    pub invitation: AnyInvitation,
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

impl From<(Request, Response)> for RequestedState {
    fn from((request, signed_response): (Request, Response)) -> RequestedState {
        trace!("ConnectionInviter: transit state to RespondedState");
        RequestedState {
            signed_response,
            did_doc: request.content.connection.did_doc,
            thread_id: request.id,
        }
    }
}
