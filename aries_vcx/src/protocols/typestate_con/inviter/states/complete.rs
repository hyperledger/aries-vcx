use std::clone::Clone;

use messages::diddoc::aries::diddoc::AriesDidDoc;
use messages::protocols::discovery::disclose::{Disclose, ProtocolDescriptor};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompleteState {
    pub did_doc: AriesDidDoc,
    pub protocols: Option<Vec<ProtocolDescriptor>>,
    pub thread_id: Option<String>,
}

impl CompleteState {
    pub fn remote_protocols(&self) -> Option<&[ProtocolDescriptor]> {
        self.protocols.as_deref()
    }

    pub fn handle_disclose(&mut self, disclose: Disclose) {
        self.protocols = Some(disclose.protocols)
    }
}

impl From<(CompleteState, Vec<ProtocolDescriptor>)> for CompleteState {
    fn from((state, protocols): (CompleteState, Vec<ProtocolDescriptor>)) -> CompleteState {
        trace!("ConnectionInviter: transit state from CompleteState to CompleteState");
        CompleteState {
            did_doc: state.did_doc,
            thread_id: state.thread_id,
            protocols: Some(protocols),
        }
    }
}
