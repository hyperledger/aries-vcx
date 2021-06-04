use crate::error::prelude::*;
use crate::aries::handlers::connection::agent_info::AgentInfo;
use crate::aries::handlers::connection::inviter::states::null::NullState;
use crate::aries::handlers::connection::inviter::states::responded::RespondedState;
use crate::aries::handlers::connection::inviter::states::requested::RequestedState;
use crate::aries::messages::connection::invite::Invitation;
use crate::aries::messages::connection::problem_report::ProblemReport;
use crate::aries::messages::connection::request::Request;
use crate::aries::messages::connection::response::{Response, SignedResponse};

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

impl From<Request> for RequestedState {
    fn from(request: Request) -> RequestedState {
        trace!("ConnectionInviter: transit state from InvitedState to RequestedState");
        let did_doc = request.clone().connection.did_doc;
        RequestedState { request, did_doc }
    }
}

impl InvitedState {
    pub fn handle_connection_request(&self,
                                     request: &Request,
                                     pw_vk: &str) -> VcxResult<(SignedResponse, AgentInfo)> {
        trace!("ConnectionInviter:handle_connection_request >>> request: {:?}", request);

        request.connection.did_doc.validate()?;

        // provision a new keys
        let new_agent_info: AgentInfo = AgentInfo::create_agent()?;

        let response = Response::create()
            .set_did(new_agent_info.pw_did.to_string())
            .set_service_endpoint(new_agent_info.agency_endpoint()?)
            .set_keys(new_agent_info.recipient_keys(), new_agent_info.routing_keys()?)
            .ask_for_ack();

        let signed_response = response.clone()
            .set_thread_id(&request.id.0)
            .encode(pw_vk)?;


        request.connection.did_doc.send_message(&signed_response.to_a2a_message(), &pw_vk)?;

        Ok((signed_response, new_agent_info))
    }
}
