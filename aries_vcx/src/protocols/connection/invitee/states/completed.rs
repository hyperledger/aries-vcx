use std::clone::Clone;

use messages::{
    diddoc::aries::diddoc::AriesDidDoc,
    protocols::discovery::disclose::{Disclose, ProtocolDescriptor},
};

use crate::protocols::connection::trait_bounds::{BootstrapDidDoc, CompletedState, TheirDidDoc, ThreadId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Completed {
    pub(crate) did_doc: AriesDidDoc,
    pub(crate) bootstrap_did_doc: AriesDidDoc,
    pub(crate) thread_id: String,
    pub(crate) protocols: Option<Vec<ProtocolDescriptor>>,
}

impl Completed {
    pub fn new(
        did_doc: AriesDidDoc,
        bootstrap_did_doc: AriesDidDoc,
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
    fn their_did_doc(&self) -> &AriesDidDoc {
        &self.did_doc
    }
}

impl BootstrapDidDoc for Completed {
    fn bootstrap_did_doc(&self) -> &AriesDidDoc {
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
        self.protocols = Some(disclose.protocols)
    }
}
