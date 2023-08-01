use self::{
    invitee::{
        state::{InviteeComplete, InviteeRequested},
        InviteeConnection,
    },
    inviter::{state::InviterComplete, InviterConnection},
};

use super::AriesSM;

pub mod invitee;
pub mod inviter;

/// Enum that can represent the possible states of the
/// state machine from the [connection protocol](<https://github.com/hyperledger/aries-rfcs/blob/main/features/0160-connection-protocol/README.md>).
#[derive(Debug, Clone)]
pub enum ConnectionSM {
    InviteeRequested(InviteeConnection<InviteeRequested>),
    InviteeComplete(InviteeConnection<InviteeComplete>),
    InviterComplete(InviterConnection<InviterComplete>),
}

impl From<InviteeConnection<InviteeRequested>> for AriesSM {
    fn from(value: InviteeConnection<InviteeRequested>) -> Self {
        Self::Connection(ConnectionSM::InviteeRequested(value))
    }
}

impl From<InviteeConnection<InviteeComplete>> for AriesSM {
    fn from(value: InviteeConnection<InviteeComplete>) -> Self {
        Self::Connection(ConnectionSM::InviteeComplete(value))
    }
}

impl From<InviterConnection<InviterComplete>> for AriesSM {
    fn from(value: InviterConnection<InviterComplete>) -> Self {
        Self::Connection(ConnectionSM::InviterComplete(value))
    }
}
