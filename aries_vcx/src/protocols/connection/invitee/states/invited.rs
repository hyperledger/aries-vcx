use messages::connection::invite::Invitation;
use messages::connection::request::Request;
use crate::protocols::connection::invitee::states::requested::RequestedState;
use messages::did_doc::DidDoc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvitedState {
    pub invitation: Invitation,
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
