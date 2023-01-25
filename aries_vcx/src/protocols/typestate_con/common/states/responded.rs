use messages::diddoc::aries::diddoc::AriesDidDoc;

use crate::protocols::typestate_con::trait_bounds::TheirDidDoc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RespondedState {
    pub did_doc: AriesDidDoc,
    pub thread_id: String
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