use crate::protocols::connection::invitee::states::invited::InvitedState;
use messages::connection::invite::Invitation;
use messages::connection::problem_report::ProblemReport;
use messages::did_doc::DidDoc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitialState {
    problem_report: Option<ProblemReport>,
    pub did_doc: Option<DidDoc>,
}

impl From<(InitialState, Invitation, DidDoc)> for InvitedState {
    fn from((_state, invitation, did_doc): (InitialState, Invitation, DidDoc)) -> InvitedState {
        trace!("ConnectionInvitee: transit state from InitialState to InvitedState");
        InvitedState { invitation, did_doc }
    }
}

impl InitialState {
    pub fn new(problem_report: Option<ProblemReport>, did_doc: Option<DidDoc>) -> Self {
        InitialState {
            problem_report,
            did_doc,
        }
    }
}
