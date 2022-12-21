use messages::did_doc::aries::diddoc::AriesDidDoc;
use messages::protocols::connection::problem_report::ProblemReport;
use messages::protocols::connection::request::Request;
use messages::protocols::connection::response::Response;
use crate::protocols::connection::invitee::states::initial::InitialState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RespondedState {
    pub response: Response,
    pub request: Request,
    pub did_doc: AriesDidDoc,
}

impl From<(RespondedState, ProblemReport)> for InitialState {
    fn from((_state, problem_report): (RespondedState, ProblemReport)) -> InitialState {
        trace!(
            "ConnectionInvitee: transit state from RespondedState to InitialState, problem_report: {:?}",
            problem_report
        );
        InitialState::new(Some(problem_report), None)
    }
}
