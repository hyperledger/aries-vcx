use messages::{diddoc::aries::diddoc::AriesDidDoc, protocols::connection::request::Request};

use crate::protocols::connection::trait_bounds::{TheirDidDoc, ThreadId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Invited {
    pub(crate) did_doc: AriesDidDoc,
    pub(crate) thread_id: String,
    pub(crate) request: Request,
}

impl Invited {
    pub fn new(did_doc: AriesDidDoc, thread_id: String, request: Request) -> Self {
        Self {
            did_doc,
            thread_id,
            request,
        }
    }
}

impl TheirDidDoc for Invited {
    fn their_did_doc(&self) -> &AriesDidDoc {
        &self.did_doc
    }
}

impl ThreadId for Invited {
    fn thread_id(&self) -> &str {
        &self.thread_id
    }
}
