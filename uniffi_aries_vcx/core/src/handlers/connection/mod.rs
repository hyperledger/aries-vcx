use aries_vcx::protocols::connection::ThinState;

pub mod connection;

/// Wraps [ThinState], as uniffi cannot process enums with un-named fields
pub struct ConnectionState {
    pub role: ConnectionRole,
    pub protocol_state: ConnectionProtocolState,
}

pub enum ConnectionRole {
    Invitee,
    Inviter,
}

pub enum ConnectionProtocolState {
    Initial,
    Invited,
    Requested,
    Responded,
    Completed,
}

impl From<ThinState> for ConnectionState {
    fn from(x: ThinState) -> Self {
        match x {
            ThinState::Inviter(state) => ConnectionState {
                role: ConnectionRole::Inviter,
                protocol_state: ConnectionProtocolState::from(state),
            },
            ThinState::Invitee(state) => ConnectionState {
                role: ConnectionRole::Invitee,
                protocol_state: ConnectionProtocolState::from(state),
            },
        }
    }
}

impl From<aries_vcx::protocols::connection::State> for ConnectionProtocolState {
    fn from(value: aries_vcx::protocols::connection::State) -> Self {
        match value {
            aries_vcx::protocols::connection::State::Initial => ConnectionProtocolState::Initial,
            aries_vcx::protocols::connection::State::Invited => ConnectionProtocolState::Invited,
            aries_vcx::protocols::connection::State::Requested => ConnectionProtocolState::Requested,
            aries_vcx::protocols::connection::State::Responded => ConnectionProtocolState::Responded,
            aries_vcx::protocols::connection::State::Completed => ConnectionProtocolState::Completed,
        }
    }
}
