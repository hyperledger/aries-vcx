use messages::{
    diddoc::aries::diddoc::AriesDidDoc,
    protocols::connection::{problem_report::ProblemReport, request::Request, response::Response},
};

use crate::protocols::mediated_connection::invitee::states::{initial::InitialState, responded::RespondedState};

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
