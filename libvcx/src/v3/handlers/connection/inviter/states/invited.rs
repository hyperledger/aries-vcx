use error::prelude::*;
use v3::handlers::connection::agent_info::AgentInfo;
use v3::handlers::connection::inviter::states::null::NullState;
use v3::handlers::connection::inviter::states::responded::RespondedState;
use v3::messages::connection::invite::Invitation;
use v3::messages::connection::problem_report::{ProblemReport};
use v3::messages::connection::request::Request;
use v3::messages::connection::response::{Response, SignedResponse};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitedState {
    pub invitation: Invitation
}

impl From<(InvitedState, ProblemReport)> for NullState {
    fn from((_state, _error): (InvitedState, ProblemReport)) -> NullState {
        trace!("ConnectionInviter: transit state from InvitedState to NullState");
        NullState {}
    }
}

impl From<(InvitedState, Request, SignedResponse, AgentInfo)> for RespondedState {
    fn from((_state, request, response, prev_agent_info): (InvitedState, Request, SignedResponse, AgentInfo)) -> RespondedState {
        trace!("ConnectionInviter: transit state from InvitedState to RespondedState");
        RespondedState { response, did_doc: request.connection.did_doc, prev_agent_info }
    }
}

impl InvitedState {
    pub fn handle_connection_request(&self, request: &Request,
                                     agent_info: &AgentInfo) -> VcxResult<(SignedResponse, AgentInfo)> {
        trace!("ConnectionInviter:handle_connection_request >>> request: {:?}, agent_info: {:?}", request, agent_info);

        request.connection.did_doc.validate()?;

        let prev_agent_info = agent_info.clone();

        // provision a new keys
        let new_agent_info: AgentInfo = agent_info.create_agent()?;

        let response = Response::create()
            .set_did(new_agent_info.pw_did.to_string())
            .set_service_endpoint(new_agent_info.agency_endpoint()?)
            .set_keys(new_agent_info.recipient_keys(), new_agent_info.routing_keys()?)
            .ask_for_ack();

        let signed_response = response.clone()
            .set_thread_id(&request.id.0)
            .encode(&prev_agent_info.pw_vk)?;

        new_agent_info.send_message(&signed_response.to_a2a_message(), &request.connection.did_doc)?;

        Ok((signed_response, new_agent_info))
    }
}
