use crate::handlers::connection::inviter::states::invited::InvitedState;
use crate::messages::connection::problem_report::ProblemReport;
use crate::messages::connection::invite::Invitation;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitialState {
    problem_report: Option<ProblemReport>
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
