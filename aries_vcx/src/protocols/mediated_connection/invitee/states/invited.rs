use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::msg_fields::protocols::connection::request::Request;

use crate::handlers::util::AnyInvitation;
use crate::protocols::mediated_connection::invitee::states::requested::RequestedState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvitedState {
    pub invitation: AnyInvitation,
    pub did_doc: AriesDidDoc,
}

impl From<(InvitedState, Request, AriesDidDoc)> for RequestedState {
    fn from((_state, request, did_doc): (InvitedState, Request, AriesDidDoc)) -> RequestedState {
        trace!("ConnectionInvitee: transit state from InvitedState to RequestedState");
        RequestedState { request, did_doc }
    }
}
