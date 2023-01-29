use std::clone::Clone;

use messages::diddoc::aries::diddoc::AriesDidDoc;
use messages::protocols::discovery::disclose::{Disclose, ProtocolDescriptor};

use crate::protocols::typestate_con::traits::{TheirDidDoc, ThreadId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompleteState {
    pub(crate) did_doc: AriesDidDoc,
    pub(crate) thread_id: String,
    pub(crate) protocols: Option<Vec<ProtocolDescriptor>>,
}

impl CompleteState {
    pub fn new(did_doc: AriesDidDoc, thread_id: String, protocols: Option<Vec<ProtocolDescriptor>>) -> Self {
        Self {
            did_doc,
            thread_id,
            protocols,
        }
    }

    pub fn remote_protocols(&self) -> Option<&[ProtocolDescriptor]> {
        self.protocols.as_deref()
    }

    pub fn handle_disclose(&mut self, disclose: Disclose) {
        self.protocols = Some(disclose.protocols)
    }
}

impl TheirDidDoc for CompleteState {
    fn their_did_doc(&self) -> &AriesDidDoc {
        &self.did_doc
    }
}

impl ThreadId for CompleteState {
    fn thread_id(&self) -> &str {
        &self.thread_id
    }
}