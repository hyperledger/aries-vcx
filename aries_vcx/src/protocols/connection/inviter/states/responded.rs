use messages::diddoc::aries::diddoc::AriesDidDoc;

use crate::protocols::connection::trait_bounds::{TheirDidDoc, ThreadId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Responded {
    pub(crate) did_doc: AriesDidDoc,
    pub(crate) thread_id: String,
}

impl Responded {
    pub fn new(did_doc: AriesDidDoc, thread_id: String) -> Self {
        Self { did_doc, thread_id }
    }
}

impl TheirDidDoc for Responded {
    fn their_did_doc(&self) -> &AriesDidDoc {
        &self.did_doc
    }
}

impl ThreadId for Responded {
    fn thread_id(&self) -> &str {
        &self.thread_id
    }
}
