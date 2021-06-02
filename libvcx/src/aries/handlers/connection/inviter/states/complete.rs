use crate::error::VcxResult;
use crate::aries::handlers::connection::agent_info::AgentInfo;
use crate::aries::handlers::connection::inviter::state_machine::InviterState;
use crate::aries::handlers::connection::util::handle_ping;
use crate::aries::messages::a2a::protocol_registry::ProtocolRegistry;
use crate::aries::messages::connection::did_doc::DidDoc;
use crate::aries::messages::discovery::disclose::{Disclose, ProtocolDescriptor};
use crate::aries::messages::discovery::query::Query;
use crate::aries::messages::trust_ping::ping::Ping;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteState {
    pub did_doc: DidDoc,
    pub protocols: Option<Vec<ProtocolDescriptor>>,
}

impl From<(CompleteState, Vec<ProtocolDescriptor>)> for CompleteState {
    fn from((state, protocols): (CompleteState, Vec<ProtocolDescriptor>)) -> CompleteState {
        trace!("ConnectionInviter: transit state from CompleteState to CompleteState");
        CompleteState { did_doc: state.did_doc, protocols: Some(protocols) }
    }
}

impl CompleteState {

    pub fn handle_send_ping(&self, comment: Option<String>, agent_info: &AgentInfo) -> VcxResult<()> {
        let ping =
            Ping::create()
                .request_response()
                .set_comment(comment);

        self.did_doc.send_message(&ping.to_a2a_message(), &agent_info.pw_vk).ok();
        Ok(())
    }

    pub fn handle_ping(&self, ping: &Ping, agent_info: &AgentInfo) -> VcxResult<()> {
        handle_ping(ping, agent_info, &self.did_doc)
    }

    pub fn handle_discover_features(&self, query: Option<String>, comment: Option<String>, agent_info: &AgentInfo) -> VcxResult<()> {
        let query_ =
            Query::create()
                .set_query(query)
                .set_comment(comment);

        self.did_doc.send_message(&query_.to_a2a_message(), &agent_info.pw_vk)
    }

    pub fn handle_discovery_query(&self, query: Query, agent_info: &AgentInfo) -> VcxResult<()> {
        let protocols = ProtocolRegistry::init().get_protocols_for_query(query.query.as_ref().map(String::as_str));

        let disclose = Disclose::create()
            .set_protocols(protocols)
            .set_thread_id(query.id.0.clone());

        self.did_doc.send_message(&disclose.to_a2a_message(), &agent_info.pw_vk)
    }
}