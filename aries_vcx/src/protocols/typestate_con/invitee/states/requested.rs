use messages::diddoc::aries::diddoc::AriesDidDoc;

use crate::protocols::typestate_con::traits::{HandleProblem, TheirDidDoc, ThreadId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestedState {
    pub(crate) did_doc: AriesDidDoc,
    pub(crate) thread_id: String,
}

impl RequestedState {
    pub fn new(did_doc: AriesDidDoc, thread_id: String) -> Self {
        Self { did_doc, thread_id }
    }
}

impl TheirDidDoc for RequestedState {
    fn their_did_doc(&self) -> &AriesDidDoc {
        &self.did_doc
    }
}

impl ThreadId for RequestedState {
    fn thread_id(&self) -> &str {
        &self.thread_id
    }
}

impl HandleProblem for RequestedState {}
