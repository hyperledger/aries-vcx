use crate::handlers::connection::invitee::states::invited::InvitedState;
use crate::messages::connection::invite::Invitation;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitialState {}

impl From<(InitialState, Invitation)> for InvitedState {
    fn from((_state, invitation): (InitialState, Invitation)) -> InvitedState {
        trace!("ConnectionInvitee: transit state from InitialState to InvitedState");
        InvitedState { invitation }
    }
}
