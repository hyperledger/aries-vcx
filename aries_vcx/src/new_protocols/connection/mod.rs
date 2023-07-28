use self::{
    invitee::{
        state::{InviteeComplete, InviteeRequested},
        InviteeConnection,
    },
    inviter::{state::InviterComplete, InviterConnection},
};

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
