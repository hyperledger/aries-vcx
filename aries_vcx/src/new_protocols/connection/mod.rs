use crate::protocols::connection::invitee::InviteeConnection;

use self::{
    invitee::state::{InviteeRequested, InviteeResponded},
    inviter::{
        state::{InviterComplete, InviterRequested, InviterResponded},
        InviterConnection,
    },
};

pub mod invitee;
pub mod inviter;

/// Enum that can represent the possible states of the
/// state machine from the [connection protocol](<https://github.com/hyperledger/aries-rfcs/blob/main/features/0160-connection-protocol/README.md>).
#[derive(Debug, Clone)]
pub enum ConnectionSM {
    InviteeRequested(InviteeConnection<InviteeRequested>),
    InviteeResponded(InviteeConnection<InviteeResponded>),
    InviteeComplete(InviteeConnection<InviteeComplete>),
    InviterRequested(InviterConnection<InviterRequested>),
    InviterResponded(InviterConnection<InviterResponded>),
    InviterComplete(InviterConnection<InviterComplete>),
}
