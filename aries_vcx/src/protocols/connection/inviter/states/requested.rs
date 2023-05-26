use did_doc::schema::did_doc::DidDocument;
use did_resolver_sov::resolution::ExtraFieldsSov;
use messages::msg_fields::protocols::connection::response::Response;

use crate::protocols::connection::trait_bounds::{TheirDidDoc, ThreadId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Requested {
    pub(crate) signed_response: Response,
    pub(crate) did_doc: DidDocument<ExtraFieldsSov>,
}

impl Requested {
    pub fn new(signed_response: Response, did_doc: DidDocument<ExtraFieldsSov>) -> Self {
        Self {
            signed_response,
            did_doc,
        }
    }
}

impl TheirDidDoc for Requested {
    fn their_did_doc(&self) -> &DidDocument<ExtraFieldsSov> {
        &self.did_doc
    }
}

impl ThreadId for Requested {
    //TODO: This should land in the threadlike macro.
    fn thread_id(&self) -> &str {
        self.signed_response.decorators.thread.thid.as_str()
    }
}
