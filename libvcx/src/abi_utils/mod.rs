use crate::aries::handlers::connection::connection::ConnectionState;
use crate::aries::handlers::connection::invitee::state_machine::InviteeState;
use crate::aries::handlers::connection::inviter::state_machine::InviterState;

#[macro_use]
pub mod ccallback;
#[macro_use]
pub mod cstring;
pub mod timeout;
pub mod runtime;

impl From<ConnectionState> for u32 {
    fn from(state: ConnectionState) -> u32 {
        match state {
            ConnectionState::Inviter(inviter_state) => {
                match inviter_state {
                    InviterState::Null => 0,
                    InviterState::Invited => 1,
                    InviterState::Requested => 2,
                    InviterState::Responded => 3,
                    InviterState::Completed => 4,
                }
            }
            ConnectionState::Invitee(invitee_state) => {
                match invitee_state {
                    InviteeState::Null => 0,
                    InviteeState::Invited => 1,
                    InviteeState::Requested => 2,
                    InviteeState::Responded => 3,
                    InviteeState::Completed => 4,
                }
            }
        }
    }
}
