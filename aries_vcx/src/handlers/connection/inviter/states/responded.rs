use crate::error::prelude::*;
use crate::handlers::connection::inviter::states::complete::CompleteState;
use crate::handlers::connection::inviter::states::initial::InitialState;
use crate::handlers::connection::util::handle_ping;
use crate::messages::a2a::A2AMessage;
use crate::messages::ack::Ack;
use crate::messages::connection::did_doc::DidDoc;
use crate::messages::connection::problem_report::ProblemReport;
use crate::messages::connection::response::SignedResponse;
use crate::messages::trust_ping::ping::Ping;
use crate::messages::trust_ping::ping_response::PingResponse;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RespondedState {
    pub signed_response: SignedResponse,
    pub did_doc: DidDoc,
}


impl From<(RespondedState, ProblemReport)> for InitialState {
    fn from((_state, problem_report): (RespondedState, ProblemReport)) -> InitialState {
        trace!("ConnectionInviter: transit state from RespondedState to InitialState");
        InitialState::new(Some(problem_report))
    }
}

impl From<(RespondedState, Ack)> for CompleteState {
    fn from((state, _ack): (RespondedState, Ack)) -> CompleteState {
        trace!("ConnectionInviter: transit state from RespondedState to CompleteState");
        CompleteState { did_doc: state.did_doc, thread_id: Some(state.signed_response.get_thread_id()),  protocols: None }
    }
}

impl From<(RespondedState, Ping)> for CompleteState {
    fn from((state, _ping): (RespondedState, Ping)) -> CompleteState {
        trace!("ConnectionInviter: transit state from RespondedState to CompleteState");
        CompleteState { did_doc: state.did_doc, thread_id: Some(state.signed_response.get_thread_id()), protocols: None }
    }
}

impl From<(RespondedState, PingResponse)> for CompleteState {
    fn from((state, _ping_response): (RespondedState, PingResponse)) -> CompleteState {
        trace!("ConnectionInviter: transit state from RespondedState to CompleteState");
        CompleteState { did_doc: state.did_doc, thread_id: Some(state.signed_response.get_thread_id()), protocols: None }
    }
}

impl RespondedState {
    pub fn handle_ping(&self,
                       ping: &Ping,
                       pw_vk: &str,
                       send_message: fn(&str, &DidDoc, &A2AMessage) -> VcxResult<()>,
    ) -> VcxResult<()> {
        handle_ping(ping, pw_vk, &self.did_doc, send_message)
    }
}
