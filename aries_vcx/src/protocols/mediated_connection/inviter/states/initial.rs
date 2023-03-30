use messages::msg_fields::protocols::connection::problem_report::ProblemReport;

use crate::handlers::util::AnyInvitation;
use crate::protocols::mediated_connection::inviter::states::invited::InvitedState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitialState {
    problem_report: Option<ProblemReport>,
}

impl From<(InitialState, AnyInvitation)> for InvitedState {
    fn from((_state, invitation): (InitialState, AnyInvitation)) -> InvitedState {
        trace!("ConnectionInviter: transit state from InitialState to InvitedState");
        InvitedState { invitation }
    }
}

impl InitialState {
    pub fn new(problem_report: Option<ProblemReport>) -> Self {
        InitialState { problem_report }
    }
}
