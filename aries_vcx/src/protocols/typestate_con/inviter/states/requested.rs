use messages::{diddoc::aries::diddoc::AriesDidDoc, protocols::connection::response::SignedResponse};

use crate::protocols::typestate_con::traits::{TheirDidDoc, ThreadId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestedState {
    pub(crate) signed_response: SignedResponse,
    pub(crate) did_doc: AriesDidDoc,
}

impl RequestedState {
    pub fn new(signed_response: SignedResponse, did_doc: AriesDidDoc) -> Self {
        Self {
            signed_response,
            did_doc,
        }
    }
}

impl TheirDidDoc for RequestedState {
    fn their_did_doc(&self) -> &AriesDidDoc {
        &self.did_doc
    }
}

impl ThreadId for RequestedState {
    //TODO: This should land in the threadlike macro.
    fn thread_id(&self) -> &str {
        self.signed_response
            .thread
            .thid
            .as_deref()
            .unwrap_or(&self.signed_response.id.0)
    }
}
