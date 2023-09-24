use std::clone::Clone;

use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::msg_fields::protocols::{
    connection::response::Response, discover_features::ProtocolDescriptor,
};

use crate::protocols::mediated_connection::invitee::states::{
    requested::RequestedState, responded::RespondedState,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletedState {
    pub did_doc: AriesDidDoc,
    pub bootstrap_did_doc: AriesDidDoc,
    pub protocols: Option<Vec<ProtocolDescriptor>>,
}

impl From<(CompletedState, Vec<ProtocolDescriptor>)> for CompletedState {
    fn from((state, protocols): (CompletedState, Vec<ProtocolDescriptor>)) -> CompletedState {
        trace!("ConnectionInvitee: transit state from CompleteState to CompleteState");
        CompletedState {
            bootstrap_did_doc: state.bootstrap_did_doc,
            did_doc: state.did_doc,
            protocols: Some(protocols),
        }
    }
}

impl From<(RequestedState, AriesDidDoc, Response)> for CompletedState {
    fn from(
        (state, did_doc, _response): (RequestedState, AriesDidDoc, Response),
    ) -> CompletedState {
        trace!("ConnectionInvitee: transit state from RequestedState to CompleteState");
        CompletedState {
            bootstrap_did_doc: state.did_doc,
            did_doc,
            protocols: None,
        }
    }
}

impl From<RespondedState> for CompletedState {
    fn from(state: RespondedState) -> CompletedState {
        trace!("ConnectionInvitee: transit state from RespondedState to CompleteState");
        CompletedState {
            bootstrap_did_doc: state.did_doc,
            did_doc: state.resp_con_data.did_doc,
            protocols: None,
        }
    }
}
