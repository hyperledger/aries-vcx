use crate::error::prelude::*;
use crate::aries::handlers::connection::agent_info::AgentInfo;
use crate::aries::handlers::connection::inviter::states::complete::CompleteState;
use crate::aries::handlers::connection::inviter::states::responded::RespondedState;
use crate::aries::handlers::connection::inviter::states::null::NullState;
use crate::aries::messages::ack::Ack;
use crate::aries::messages::connection::did_doc::DidDoc;
use crate::aries::messages::connection::problem_report::ProblemReport;
use crate::aries::messages::connection::request::Request;
use crate::aries::messages::connection::response::{Response, SignedResponse};
use crate::aries::messages::trust_ping::ping::Ping;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestedState {
    pub request: Request,
    pub did_doc: DidDoc,
}

impl From<(RequestedState, ProblemReport)> for NullState {
    fn from((_state, _error): (RequestedState, ProblemReport)) -> NullState {
        trace!("ConnectionInviter: transit state from RequestedState to NullState");
        NullState {}
    }
}

impl From<(RequestedState, SignedResponse, AgentInfo)> for RespondedState {
    fn from((state, response, prev_agent_info): (RequestedState, SignedResponse, AgentInfo)) -> RespondedState {
        trace!("ConnectionInviter: transit state from RequestedState to RespondedState");
        RespondedState { response, did_doc: state.did_doc, prev_agent_info }
    }
}
