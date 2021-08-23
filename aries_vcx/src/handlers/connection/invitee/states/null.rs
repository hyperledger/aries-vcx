use crate::handlers::connection::invitee::states::invited::InvitedState;
use crate::messages::connection::invite::{Invitation, PairwiseInvitation, PublicInvitation};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullState {}

impl From<(NullState, PairwiseInvitation)> for InvitedState {
    fn from((_state, invitation): (NullState, PairwiseInvitation)) -> InvitedState {
        trace!("ConnectionInvitee: transit state from NullState to InvitedState");
        InvitedState { invitation: Invitation::Pairwise(invitation) }
    }
}

impl From<(NullState, PublicInvitation)> for InvitedState {
    fn from((_state, invitation): (NullState, PublicInvitation)) -> InvitedState {
        trace!("ConnectionInvitee: transit state from NullState to InvitedState");
        InvitedState { invitation: Invitation::Public(invitation) }
    }
}
