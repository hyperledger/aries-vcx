use crate::error::prelude::*;
use crate::aries::handlers::connection::pairwise_info::PairwiseInfo;
use crate::aries::handlers::connection::invitee::states::complete::CompleteState;
use crate::aries::handlers::connection::invitee::states::requested::RequestedState;
use crate::aries::handlers::connection::invitee::states::null::NullState;
use crate::aries::messages::ack::Ack;
use crate::aries::messages::connection::did_doc::DidDoc;
use crate::aries::messages::connection::problem_report::ProblemReport;
use crate::aries::messages::connection::request::Request;
use crate::aries::messages::connection::response::{Response, SignedResponse};
use crate::aries::messages::trust_ping::ping::Ping;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespondedState {
    pub response: SignedResponse,
    pub request: Request,
    pub did_doc: DidDoc,
}

impl From<(RespondedState, ProblemReport)> for NullState {
    fn from((_state, _error): (RespondedState, ProblemReport)) -> NullState {
        trace!("ConnectionInvitee: transit state from RespondedState to NullState");
        NullState {}
    }
}
