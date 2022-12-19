use messages::protocols::connection::invite::Invitation;
use messages::protocols::connection::request::Request;
use messages::did_doc::DidDoc;
use crate::protocols::connection::invitee::states::requested::RequestedState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvitedState {
    pub invitation: Invitation,
    pub did_doc: DidDoc
}

impl From<(InvitedState, Request, DidDoc)> for RequestedState {
    fn from((_state, request, did_doc): (InvitedState, Request, DidDoc)) -> RequestedState {
        trace!("ConnectionInvitee: transit state from InvitedState to RequestedState");
        RequestedState {
            request,
            did_doc
        }
    }
}
