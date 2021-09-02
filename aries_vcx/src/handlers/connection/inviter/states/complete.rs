use crate::error::VcxResult;
use crate::handlers::connection::util::handle_ping;
use crate::messages::a2a::A2AMessage;
use crate::messages::a2a::protocol_registry::ProtocolRegistry;
use crate::messages::connection::did_doc::DidDoc;
use crate::messages::discovery::disclose::{Disclose, ProtocolDescriptor};
use crate::messages::discovery::query::Query;
use crate::messages::trust_ping::ping::Ping;

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
