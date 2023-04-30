use std::clone::Clone;

use diddoc::aries::diddoc::AriesDidDoc;
use messages::msg_fields::protocols::discover_features::ProtocolDescriptor;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletedState {
    pub did_doc: AriesDidDoc,
    pub protocols: Option<Vec<ProtocolDescriptor>>,
    pub thread_id: Option<String>,
}

impl From<(CompletedState, Vec<ProtocolDescriptor>)> for CompletedState {
    fn from((state, protocols): (CompletedState, Vec<ProtocolDescriptor>)) -> CompletedState {
        trace!("ConnectionInviter: transit state from CompleteState to CompleteState");
        CompletedState {
            did_doc: state.did_doc,
            thread_id: state.thread_id,
            protocols: Some(protocols),
        }
    }
}
