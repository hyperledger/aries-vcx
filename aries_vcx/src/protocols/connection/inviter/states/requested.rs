use messages::{diddoc::aries::diddoc::AriesDidDoc, protocols::connection::response::SignedResponse};

use crate::protocols::connection::trait_bounds::{TheirDidDoc, ThreadId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Requested {
    pub(crate) signed_response: SignedResponse,
    pub(crate) did_doc: AriesDidDoc,
}

impl Requested {
    pub fn new(signed_response: SignedResponse, did_doc: AriesDidDoc) -> Self {
        Self {
            signed_response,
            did_doc,
        }
    }
}

impl TheirDidDoc for Requested {
    fn their_did_doc(&self) -> &AriesDidDoc {
        &self.did_doc
    }
}

impl ThreadId for Requested {
    //TODO: This should land in the threadlike macro.
    fn thread_id(&self) -> &str {
        self.signed_response
            .thread
            .thid
            .as_deref()
            .unwrap_or(&self.signed_response.id.0)
    }
}
