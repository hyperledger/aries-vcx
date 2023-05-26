use did_doc::schema::did_doc::DidDocument;
use did_resolver_sov::resolution::ExtraFieldsSov;

use crate::protocols::connection::trait_bounds::{BootstrapDidDoc, HandleProblem, TheirDidDoc, ThreadId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Requested {
    pub(crate) did_doc: DidDocument<ExtraFieldsSov>,
    pub(crate) thread_id: String,
}

impl Requested {
    pub fn new(did_doc: DidDocument<ExtraFieldsSov>, thread_id: String) -> Self {
        Self { did_doc, thread_id }
    }
}

impl TheirDidDoc for Requested {
    fn their_did_doc(&self) -> &DidDocument<ExtraFieldsSov> {
        &self.did_doc
    }
}

impl BootstrapDidDoc for Requested {}

impl ThreadId for Requested {
    fn thread_id(&self) -> &str {
        &self.thread_id
    }
}

impl HandleProblem for Requested {}
