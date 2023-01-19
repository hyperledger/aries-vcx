use crate::protocols::connection::invitee::states::requested::RequestedState;
use crate::protocols::connection::trait_bounds::TheirDidDoc;
use messages::diddoc::aries::diddoc::AriesDidDoc;
use messages::protocols::connection::invite::Invitation;
use messages::protocols::connection::request::Request;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvitedState {
    pub invitation: Invitation,
    pub did_doc: AriesDidDoc,
}

impl From<(InvitedState, Request, AriesDidDoc)> for RequestedState {
    fn from((_state, request, did_doc): (InvitedState, Request, AriesDidDoc)) -> RequestedState {
        trace!("ConnectionInvitee: transit state from InvitedState to RequestedState");
        RequestedState { request, did_doc }
    }
}

impl TheirDidDoc for InvitedState {
    fn their_did_doc(&self) -> &AriesDidDoc {
        &self.did_doc
    }
}