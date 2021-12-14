use crate::handlers::connection::invitee::states::initial::InitialState;
use crate::messages::connection::did_doc::DidDoc;
use crate::messages::connection::problem_report::ProblemReport;
use crate::messages::connection::request::Request;
use crate::messages::connection::response::SignedResponse;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RespondedState {
    pub response: SignedResponse,
    pub request: Request,
    pub did_doc: DidDoc,
}

impl From<(RespondedState, ProblemReport)> for InitialState {
    fn from((_state, problem_report): (RespondedState, ProblemReport)) -> InitialState {
        trace!("ConnectionInvitee: transit state from RespondedState to InitialState, problem_report: {:?}", problem_report);
        InitialState::new(Some(problem_report))
    }
}
