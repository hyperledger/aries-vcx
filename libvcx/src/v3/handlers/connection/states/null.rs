use v3::messages::connection::invite::Invitation;
use v3::handlers::connection::states::invited::InvitedState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullState {}

impl From<(NullState, Invitation)> for InvitedState {
    fn from((_state, invitation): (NullState, Invitation)) -> InvitedState {
        trace!("DidExchangeStateSM: transit state from NullState to InvitedState");
        InvitedState { invitation }
    }
}
