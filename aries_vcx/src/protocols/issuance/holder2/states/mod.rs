use self::{ack_prepared::AckPrepared, finished::Finished};

use super::Holder;

pub mod ack_prepared;
pub mod failed;
pub mod finished;
pub mod offer_received;
pub mod proposal_prepared;
pub mod request_prepared;

pub enum HolderReceiveCredentialNextState {
    Finished(Holder<Finished>),
    AckPrepared(Holder<AckPrepared>),
}
