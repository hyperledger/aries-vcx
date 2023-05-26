use did_doc::schema::did_doc::DidDocument;
use did_resolver_sov::resolution::ExtraFieldsSov;

use crate::protocols::connection::trait_bounds::{TheirDidDoc, ThreadId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Responded {
    pub(crate) did_doc: DidDocument<ExtraFieldsSov>,
    pub(crate) thread_id: String,
}

impl Responded {
    pub fn new(did_doc: DidDocument<ExtraFieldsSov>, thread_id: String) -> Self {
        Self { did_doc, thread_id }
    }
}

impl TheirDidDoc for Responded {
    fn their_did_doc(&self) -> &DidDocument<ExtraFieldsSov> {
        &self.did_doc
    }
}

impl ThreadId for Responded {
    fn thread_id(&self) -> &str {
        &self.thread_id
    }
}
