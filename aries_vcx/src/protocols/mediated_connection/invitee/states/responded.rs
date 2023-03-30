use diddoc::aries::diddoc::AriesDidDoc;
use messages2::msg_fields::protocols::connection::{problem_report::ProblemReport, request::Request, ConnectionData};

use crate::protocols::mediated_connection::invitee::states::initial::InitialState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RespondedState {
    pub resp_con_data: ConnectionData,
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
