use crate::aries::handlers::connection::inviter::states::complete::CompleteState;
use crate::aries::handlers::connection::inviter::states::null::NullState;
use crate::aries::handlers::connection::util::handle_ping;
use crate::aries::messages::ack::Ack;
use crate::aries::messages::connection::did_doc::DidDoc;
use crate::aries::messages::connection::problem_report::ProblemReport;
use crate::aries::messages::connection::response::SignedResponse;
use crate::aries::messages::trust_ping::ping::Ping;
use crate::aries::messages::trust_ping::ping_response::PingResponse;
use crate::error::prelude::*;
use crate::aries::messages::a2a::A2AMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespondedState {
    pub signed_response: SignedResponse,
    pub did_doc: DidDoc,
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
    pub fn handle_ping(&self,
                       ping: &Ping,
                       pw_vk: &str,
                       send_message: fn(&str, &DidDoc, &A2AMessage) -> VcxResult<()>
    ) -> VcxResult<()> {
        handle_ping(ping, pw_vk, &self.did_doc, send_message)
    }

}
