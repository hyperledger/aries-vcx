pub mod connection;

use aries_vcx::handlers::connection::connection::ConnectionState as VcxConnectionState;
pub enum ConnectionState {
    Initial,
    Invited,
    Requested,
    Responded,
    Completed,
}

impl From<VcxConnectionState> for ConnectionState {
    fn from(x: VcxConnectionState) -> Self {
        match x {
            VcxConnectionState::Inviter(x) => match x {
                aries_vcx::protocols::connection::inviter::state_machine::InviterState::Initial => {
                    ConnectionState::Initial
                }
                aries_vcx::protocols::connection::inviter::state_machine::InviterState::Invited => {
                    ConnectionState::Invited
                }
                aries_vcx::protocols::connection::inviter::state_machine::InviterState::Requested => {
                    ConnectionState::Requested
                }
                aries_vcx::protocols::connection::inviter::state_machine::InviterState::Responded => {
                    ConnectionState::Responded
                }
                aries_vcx::protocols::connection::inviter::state_machine::InviterState::Completed => {
                    ConnectionState::Completed
                }
            },
            VcxConnectionState::Invitee(x) => match x {
                aries_vcx::protocols::connection::invitee::state_machine::InviteeState::Initial => {
                    ConnectionState::Initial
                }
                aries_vcx::protocols::connection::invitee::state_machine::InviteeState::Invited => {
                    ConnectionState::Invited
                }
                aries_vcx::protocols::connection::invitee::state_machine::InviteeState::Requested => {
                    ConnectionState::Requested
                }
                aries_vcx::protocols::connection::invitee::state_machine::InviteeState::Responded => {
                    ConnectionState::Responded
                }
                aries_vcx::protocols::connection::invitee::state_machine::InviteeState::Completed => {
                    ConnectionState::Completed
                }
            },
        }
    }
}
