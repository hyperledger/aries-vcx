use crate::protocols::connection::inviter::states::invited::InvitedState;
use messages::connection::invite::Invitation;
use messages::connection::problem_report::ProblemReport;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitialState {
    problem_report: Option<ProblemReport>,
}

impl From<(InitialState, Invitation)> for InvitedState {
    fn from((_state, invitation): (InitialState, Invitation)) -> InvitedState {
        trace!("ConnectionInviter: transit state from InitialState to InvitedState");
        InvitedState { invitation }
    }
}

impl InitialState {
    pub fn new(problem_report: Option<ProblemReport>) -> Self {
        InitialState { problem_report }
    }
}
