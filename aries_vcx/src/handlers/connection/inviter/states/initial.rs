use crate::handlers::connection::inviter::states::invited::InvitedState;
use crate::messages::connection::invite::Invitation;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitialState {}

impl From<(InitialState, Invitation)> for InvitedState {
    fn from((_state, invitation): (InitialState, Invitation)) -> InvitedState {
        trace!("ConnectionInviter: transit state from InitialState to InvitedState");
        InvitedState { invitation }
    }
}
