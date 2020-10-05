use error::prelude::*;
use aries::handlers::connection::agent_info::AgentInfo;
use aries::handlers::connection::invitee::states::complete::CompleteState;
use aries::handlers::connection::invitee::states::null::NullState;
use aries::messages::ack::Ack;
use aries::messages::connection::did_doc::DidDoc;
use aries::messages::connection::problem_report::ProblemReport;
use aries::messages::connection::request::Request;
use aries::messages::connection::response::{Response, SignedResponse};
use aries::messages::trust_ping::ping::Ping;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestedState {
    pub request: Request,
    pub did_doc: DidDoc,
}


impl From<(RequestedState, ProblemReport)> for NullState {
    fn from((_state, _error): (RequestedState, ProblemReport)) -> NullState {
        trace!("ConnectionInvitee: transit state from RequestedState to NullState");
        NullState {}
    }
}

impl From<(RequestedState, Response)> for CompleteState {
    fn from((_state, response): (RequestedState, Response)) -> CompleteState {
        trace!("ConnectionInvitee: transit state from RequestedState to CompleteState");
        CompleteState { did_doc: response.connection.did_doc, protocols: None }
    }
}

impl RequestedState {
    pub fn handle_connection_response(&self, response: SignedResponse, agent_info: &AgentInfo) -> VcxResult<Response> {
        trace!("ConnectionInvitee:handle_connection_response >>> response: {:?}, agent_info: {:?}", response, agent_info);

        let remote_vk: String = self.did_doc.recipient_keys().get(0).cloned()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "Cannot handle Response: Remote Verkey not found"))?;

        let response: Response = response.decode(&remote_vk)?;

        if !response.from_thread(&self.request.id.0) {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot handle Response: thread id does not match: {:?}", response.thread)));
        }

        let message = if response.please_ack.is_some() {
            Ack::create()
                .set_thread_id(&response.thread.thid.clone().unwrap_or_default())
                .to_a2a_message()
        } else {
            Ping::create()
                .set_thread_id(response.thread.thid.clone().unwrap_or_default())
                .to_a2a_message()
        };

        agent_info.send_message(&message, &response.connection.did_doc)?;

        Ok(response)
    }
}
