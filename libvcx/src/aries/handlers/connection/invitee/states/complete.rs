use crate::aries::handlers::connection::invitee::states::requested::RequestedState;
use crate::aries::handlers::connection::invitee::states::responded::RespondedState;
use crate::aries::handlers::connection::util::handle_ping;
use crate::aries::messages::a2a::A2AMessage;
use crate::aries::messages::a2a::protocol_registry::ProtocolRegistry;
use crate::aries::messages::connection::did_doc::DidDoc;
use crate::aries::messages::connection::response::Response;
use crate::aries::messages::discovery::disclose::{Disclose, ProtocolDescriptor};
use crate::aries::messages::discovery::query::Query;
use crate::aries::messages::trust_ping::ping::Ping;
use crate::error::VcxResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteState {
    pub did_doc: DidDoc,
    pub protocols: Option<Vec<ProtocolDescriptor>>,
}

impl From<(CompleteState, Vec<ProtocolDescriptor>)> for CompleteState {
    fn from((state, protocols): (CompleteState, Vec<ProtocolDescriptor>)) -> CompleteState {
        trace!("ConnectionInvitee: transit state from CompleteState to CompleteState");
        CompleteState { did_doc: state.did_doc, protocols: Some(protocols) }
    }
}

impl From<(RequestedState, Response)> for CompleteState {
    fn from((_state, response): (RequestedState, Response)) -> CompleteState {
        trace!("ConnectionInvitee: transit state from RequestedState to CompleteState");
        CompleteState { did_doc: response.connection.did_doc, protocols: None }
    }
}

impl From<(RespondedState, Response)> for CompleteState {
    fn from((_state, response): (RespondedState, Response)) -> CompleteState {
        trace!("ConnectionInvitee: transit state from RespondedState to CompleteState");
        CompleteState { did_doc: response.connection.did_doc, protocols: None }
    }
}

impl CompleteState {
    pub fn handle_send_ping(&self,
                            comment: Option<String>,
                            pw_vk: &str,
                            send_message: fn(&str, &DidDoc, &A2AMessage) -> VcxResult<()>,
    ) -> VcxResult<()> {
        let ping =
            Ping::create()
                .request_response()
                .set_comment(comment);

        send_message(pw_vk, &self.did_doc, &ping.to_a2a_message()).ok();
        Ok(())
    }

    pub fn handle_ping(&self,
                       ping: &Ping,
                       pw_vk: &str,
                       send_message: fn(&str, &DidDoc, &A2AMessage) -> VcxResult<()>,
    ) -> VcxResult<()> {
        handle_ping(ping, pw_vk, &self.did_doc, send_message)
    }

    pub fn handle_discover_features(&self,
                                    query: Option<String>,
                                    comment: Option<String>,
                                    pw_vk: &str,
                                    send_message: fn(&str, &DidDoc, &A2AMessage) -> VcxResult<()>,
    ) -> VcxResult<()> {
        let query_ =
            Query::create()
                .set_query(query)
                .set_comment(comment);
        send_message(pw_vk, &self.did_doc, &query_.to_a2a_message())
    }

    pub fn handle_discovery_query(&self,
                                  query: Query,
                                  pw_vk: &str,
                                  send_message: fn(&str, &DidDoc, &A2AMessage) -> VcxResult<()>,
    ) -> VcxResult<()> {
        let protocols = ProtocolRegistry::init().get_protocols_for_query(query.query.as_ref().map(String::as_str));

        let disclose = Disclose::create()
            .set_protocols(protocols)
            .set_thread_id(query.id.0.clone());

        send_message(pw_vk, &self.did_doc, &disclose.to_a2a_message())
    }
}
