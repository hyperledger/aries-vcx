pub mod handlers;
pub mod state;

use diddoc_legacy::aries::diddoc::AriesDidDoc;

use self::state::InviterComplete;

// If this flies with a single state then
// we can remove the generic
#[derive(Clone, Debug)]
pub struct InviterConnection<S> {
    did: String,
    verkey: String,
    state: S,
}

impl InviterConnection<InviterComplete> {
    pub fn new_inviter(did: String, verkey: String, did_doc: AriesDidDoc) -> InviterConnection<InviterComplete> {
        InviterConnection {
            did,
            verkey,
            state: InviterComplete { did_doc },
        }
    }
}
