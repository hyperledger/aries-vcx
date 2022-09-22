use std::clone::Clone;

use messages::did_doc::DidDoc;
use messages::discovery::disclose::ProtocolDescriptor;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompleteState {
    pub did_doc: DidDoc,
    pub protocols: Option<Vec<ProtocolDescriptor>>,
    pub thread_id: Option<String>,
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
