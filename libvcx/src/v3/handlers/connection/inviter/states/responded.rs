use error::prelude::*;
use v3::handlers::connection::agent_info::AgentInfo;
use v3::handlers::connection::inviter::states::complete::CompleteState;
use v3::handlers::connection::inviter::states::null::NullState;
use v3::handlers::connection::util::handle_ping;
use v3::messages::ack::Ack;
use v3::messages::connection::did_doc::DidDoc;
use v3::messages::connection::problem_report::ProblemReport;
use v3::messages::connection::response::SignedResponse;
use v3::messages::trust_ping::ping::Ping;
use v3::messages::trust_ping::ping_response::PingResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespondedState {
    pub response: SignedResponse,
    pub did_doc: DidDoc,
    pub prev_agent_info: AgentInfo,
}


impl From<(RespondedState, ProblemReport)> for NullState {
    fn from((_state, _error): (RespondedState, ProblemReport)) -> NullState {
        trace!("ConnectionInviter: transit state from RespondedState to NullState");
        NullState {}
    }
}

impl From<(RespondedState, Ack)> for CompleteState {
    fn from((state, _ack): (RespondedState, Ack)) -> CompleteState {
        trace!("ConnectionInviter: transit state from RespondedState to CompleteState");
        CompleteState { did_doc: state.did_doc, protocols: None }
    }
}

impl From<(RespondedState, Ping)> for CompleteState {
    fn from((state, _ping): (RespondedState, Ping)) -> CompleteState {
        trace!("ConnectionInviter: transit state from RespondedState to CompleteState");
        CompleteState { did_doc: state.did_doc, protocols: None }
    }
}

impl From<(RespondedState, PingResponse)> for CompleteState {
    fn from((state, _ping_response): (RespondedState, PingResponse)) -> CompleteState {
        trace!("ConnectionInviter: transit state from RespondedState to CompleteState");
        CompleteState { did_doc: state.did_doc, protocols: None }
    }
}

impl RespondedState {
    pub fn handle_ping(&self, ping: &Ping, agent_info: &AgentInfo) -> VcxResult<()> {
        handle_ping(ping, agent_info, &self.did_doc)
    }
}
