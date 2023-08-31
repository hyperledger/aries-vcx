use super::{GenericState, InviteeState, InviterState};

/// Small sized enum used for determining
/// a connection's state in terms of initiation type.
#[derive(Clone, Copy, Debug)]
pub enum ThinState {
    Invitee(State),
    Inviter(State),
}

/// Small sized enum used for determining
/// a connection's state in terms of connection stage.
#[derive(Clone, Copy, Debug)]
pub enum State {
    Initial,
    Invited,
    Requested,
    Responded,
    Completed,
}

impl From<&GenericState> for ThinState {
    fn from(value: &GenericState) -> Self {
        match value {
            GenericState::Invitee(v) => Self::Invitee(v.into()),
            GenericState::Inviter(v) => Self::Inviter(v.into()),
        }
    }
}

impl From<&InviterState> for State {
    fn from(value: &InviterState) -> Self {
        match value {
            InviterState::Initial(_) => Self::Initial,
            InviterState::Invited(_) => Self::Invited,
            InviterState::Requested(_) => Self::Requested,
            InviterState::Completed(_) => Self::Completed,
        }
    }
}

impl From<&InviteeState> for State {
    fn from(value: &InviteeState) -> Self {
        match value {
            InviteeState::Initial(_) => Self::Initial,
            InviteeState::Invited(_) => Self::Invited,
            InviteeState::Requested(_) => Self::Requested,
            InviteeState::Completed(_) => Self::Completed,
        }
    }
}
