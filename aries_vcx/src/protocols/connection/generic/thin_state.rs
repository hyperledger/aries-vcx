use super::{GenericState, InviteeState, InviterState};

/// Small sized enum used for determining
/// a connection's state in terms of initiation type.
#[derive(Clone, Copy, Debug)]
pub enum State {
    Invitee(ThinState),
    Inviter(ThinState),
}

/// Small sized enum used for determining
/// a connection's state in terms of connection stage.
#[derive(Clone, Copy, Debug)]
pub enum ThinState {
    Initial,
    Invited,
    Requested,
    Responded,
    Complete,
}

impl From<&GenericState> for State {
    fn from(value: &GenericState) -> Self {
        match value {
            GenericState::Invitee(v) => Self::Invitee(v.into()),
            GenericState::Inviter(v) => Self::Inviter(v.into()),
        }
    }
}

impl From<&InviterState> for ThinState {
    fn from(value: &InviterState) -> Self {
        match value {
            InviterState::Initial(_) => Self::Initial,
            InviterState::Invited(_) => Self::Invited,
            InviterState::Requested(_) => Self::Requested,
            InviterState::Responded(_) => Self::Responded,
            InviterState::Complete(_) => Self::Complete,
        }
    }
}

impl From<&InviteeState> for ThinState {
    fn from(value: &InviteeState) -> Self {
        match value {
            InviteeState::Initial(_) => Self::Initial,
            InviteeState::Invited(_) => Self::Invited,
            InviteeState::Requested(_) => Self::Requested,
            InviteeState::Responded(_) => Self::Responded,
            InviteeState::Complete(_) => Self::Complete,
        }
    }
}
