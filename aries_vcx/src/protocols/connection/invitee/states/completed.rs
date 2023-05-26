use std::clone::Clone;

use did_doc::schema::did_doc::DidDocument;
use did_resolver_sov::resolution::ExtraFieldsSov;
use messages::msg_fields::protocols::discover_features::{disclose::Disclose, ProtocolDescriptor};

use crate::protocols::connection::trait_bounds::{BootstrapDidDoc, CompletedState, TheirDidDoc, ThreadId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Completed {
    pub(crate) did_doc: DidDocument<ExtraFieldsSov>,
    pub(crate) bootstrap_did_doc: DidDocument<ExtraFieldsSov>,
    pub(crate) thread_id: String,
    pub(crate) protocols: Option<Vec<ProtocolDescriptor>>,
}

impl Completed {
    pub fn new(
        did_doc: DidDocument<ExtraFieldsSov>,
        bootstrap_did_doc: DidDocument<ExtraFieldsSov>,
        thread_id: String,
        protocols: Option<Vec<ProtocolDescriptor>>,
    ) -> Self {
        Self {
            did_doc,
            bootstrap_did_doc,
            thread_id,
            protocols,
        }
    }
}

impl TheirDidDoc for Completed {
    fn their_did_doc(&self) -> &DidDocument<ExtraFieldsSov> {
        &self.did_doc
    }
}

impl BootstrapDidDoc for Completed {
    fn bootstrap_did_doc(&self) -> &DidDocument<ExtraFieldsSov> {
        &self.bootstrap_did_doc
    }
}

impl ThreadId for Completed {
    fn thread_id(&self) -> &str {
        &self.thread_id
    }
}

impl CompletedState for Completed {
    fn remote_protocols(&self) -> Option<&[ProtocolDescriptor]> {
        self.protocols.as_deref()
    }

    fn handle_disclose(&mut self, disclose: Disclose) {
        self.protocols = Some(disclose.content.protocols)
    }
}
