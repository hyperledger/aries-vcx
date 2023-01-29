use messages::{diddoc::aries::diddoc::AriesDidDoc, protocols::connection::request::Request};

use crate::protocols::typestate_con::traits::{TheirDidDoc, ThreadId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestedState {
    pub(crate) request: Request,
}

impl RequestedState {
    pub fn new(request: Request) -> Self {
        Self { request }
    }
}

impl TheirDidDoc for RequestedState {
    fn their_did_doc(&self) -> &AriesDidDoc {
        &self.request.connection.did_doc
    }
}

impl ThreadId for RequestedState {
    //TODO: This should land in the threadlike macro.
    fn thread_id(&self) -> &str {
        self.request
            .thread
            .as_ref()
            .and_then(|t| t.thid.as_deref())
            .unwrap_or(&self.request.id.0)
    }
}
