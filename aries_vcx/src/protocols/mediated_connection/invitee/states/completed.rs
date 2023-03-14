use std::clone::Clone;

use messages::{
    diddoc::aries::diddoc::AriesDidDoc,
    protocols::{connection::response::Response, discovery::disclose::ProtocolDescriptor},
};

use crate::protocols::mediated_connection::invitee::states::{requested::RequestedState, responded::RespondedState};

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

impl From<(RequestedState, Response)> for CompletedState {
    fn from((state, response): (RequestedState, Response)) -> CompletedState {
        trace!("ConnectionInvitee: transit state from RequestedState to CompleteState");
        CompletedState {
            bootstrap_did_doc: state.did_doc,
            did_doc: response.connection.did_doc,
            protocols: None,
        }
    }
}

impl From<RespondedState> for CompletedState {
    fn from(state: RespondedState) -> CompletedState {
        trace!("ConnectionInvitee: transit state from RespondedState to CompleteState");
        CompletedState {
            bootstrap_did_doc: state.did_doc,
            did_doc: state.response.connection.did_doc,
            protocols: None,
        }
    }
}
