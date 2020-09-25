use error::VcxResult;
use v3::handlers::connection::agent_info::AgentInfo;
use v3::handlers::connection::messages::DidExchangeMessages;
use v3::handlers::connection::state_machine::DidExchangeState;
use v3::handlers::connection::states::util::handle_ping;
use v3::messages::a2a::protocol_registry::ProtocolRegistry;
use v3::messages::connection::did_doc::DidDoc;
use v3::messages::discovery::disclose::{Disclose, ProtocolDescriptor};
use v3::messages::discovery::query::Query;
use v3::messages::trust_ping::ping::Ping;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteState {
    pub did_doc: DidDoc,
    pub protocols: Option<Vec<ProtocolDescriptor>>,
}

impl From<(CompleteState, Vec<ProtocolDescriptor>)> for CompleteState {
    fn from((state, protocols): (CompleteState, Vec<ProtocolDescriptor>)) -> CompleteState {
        trace!("DidExchangeStateSM: transit state from CompleteState to CompleteState");
        CompleteState { did_doc: state.did_doc, protocols: Some(protocols) }
    }
}

impl CompleteState {
    pub fn handle_message(self, message: DidExchangeMessages, agent_info: &AgentInfo) -> VcxResult<DidExchangeState> {
        Ok(match message {
            DidExchangeMessages::SendPing(comment) => {
                self.handle_send_ping(comment, agent_info)?;
                DidExchangeState::Completed(self)
            }
            DidExchangeMessages::PingReceived(ping) => {
                self.handle_ping(&ping, agent_info)?;
                DidExchangeState::Completed(self)
            }
            DidExchangeMessages::PingResponseReceived(_) => {
                DidExchangeState::Completed(self)
            }
            DidExchangeMessages::DiscoverFeatures((query_, comment)) => {
                self.handle_discover_features(query_, comment, agent_info)?;
                DidExchangeState::Completed(self)
            }
            DidExchangeMessages::QueryReceived(query) => {
                self.handle_discovery_query(query, agent_info)?;
                DidExchangeState::Completed(self)
            }
            DidExchangeMessages::DiscloseReceived(disclose) => {
                DidExchangeState::Completed((self, disclose.protocols).into())
            }
            _ => {
                DidExchangeState::Completed(self)
            }
        })
    }

    fn handle_send_ping(&self, comment: Option<String>, agent_info: &AgentInfo) -> VcxResult<()> {
        let ping =
            Ping::create()
                .request_response()
                .set_comment(comment);

        agent_info.send_message(&ping.to_a2a_message(), &self.did_doc).ok();
        Ok(())
    }

    fn handle_ping(&self, ping: &Ping, agent_info: &AgentInfo) -> VcxResult<()> {
        handle_ping(ping, agent_info, &self.did_doc)
    }

    fn handle_discover_features(&self, query: Option<String>, comment: Option<String>, agent_info: &AgentInfo) -> VcxResult<()> {
        let query_ =
            Query::create()
                .set_query(query)
                .set_comment(comment);

        agent_info.send_message(&query_.to_a2a_message(), &self.did_doc)
    }

    fn handle_discovery_query(&self, query: Query, agent_info: &AgentInfo) -> VcxResult<()> {
        let protocols = ProtocolRegistry::init().get_protocols_for_query(query.query.as_ref().map(String::as_str));

        let disclose = Disclose::create()
            .set_protocols(protocols)
            .set_thread_id(query.id.0.clone());

        agent_info.send_message(&disclose.to_a2a_message(), &self.did_doc)
    }
}