use messages::diddoc::aries::diddoc::AriesDidDoc;

pub(super) mod complete;
pub(super) mod initial;
pub(super) mod invited;
pub(super) mod requested;
pub(super) mod responded;

/// Marker trait for invitee states. 
/// Used for type bounds and common state methods
pub trait InviteeState {
    fn their_did_doc(&self) -> Option<AriesDidDoc>;

    /// Default implementation is to return the state's `DidDoc`.
    fn bootstrap_did_doc(&self) -> Option<AriesDidDoc> {
        self.their_did_doc()
    }
}