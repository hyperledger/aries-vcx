use messages::diddoc::aries::diddoc::AriesDidDoc;

use crate::protocols::connection::traits::{TheirDidDoc, ThreadId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RespondedState {
    pub(crate) did_doc: AriesDidDoc,
    pub(crate) thread_id: String,
}

impl RespondedState {
    pub fn new(did_doc: AriesDidDoc, thread_id: String) -> Self {
        Self { did_doc, thread_id }
    }
}

impl TheirDidDoc for RespondedState {
    fn their_did_doc(&self) -> &AriesDidDoc {
        &self.did_doc
    }
}

impl ThreadId for RespondedState {
    fn thread_id(&self) -> &str {
        &self.thread_id
    }
}
