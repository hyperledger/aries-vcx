use messages::{protocols::connection::request::Request, diddoc::aries::diddoc::AriesDidDoc};

use crate::protocols::typestate_con::trait_bounds::TheirDidDoc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestedState {
    pub request: Request,
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
