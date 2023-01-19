use std::clone::Clone;

use crate::protocols::connection::invitee::states::requested::RequestedState;
use crate::protocols::connection::invitee::states::responded::RespondedState;
use crate::protocols::connection::trait_bounds::{TheirDidDoc};
use messages::diddoc::aries::diddoc::AriesDidDoc;
use messages::protocols::connection::response::Response;
use messages::protocols::discovery::disclose::{ProtocolDescriptor, Disclose};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompleteState {
    pub did_doc: AriesDidDoc,
    pub bootstrap_did_doc: AriesDidDoc,
    pub protocols: Option<Vec<ProtocolDescriptor>>,
}

impl CompleteState {
   pub fn remote_protocols(&self) -> Option<&[ProtocolDescriptor]> {
        self.protocols.as_deref()
    }

    pub fn handle_disclose(&mut self, disclose: Disclose) {
        self.protocols = Some(disclose.protocols)
    }
}

impl From<(CompleteState, Vec<ProtocolDescriptor>)> for CompleteState {
    fn from((state, protocols): (CompleteState, Vec<ProtocolDescriptor>)) -> CompleteState {
        trace!("ConnectionInvitee: transit state from CompleteState to CompleteState");
        CompleteState {
            bootstrap_did_doc: state.bootstrap_did_doc,
            did_doc: state.did_doc,
            protocols: Some(protocols),
        }
    }
}

impl From<(RequestedState, Response)> for CompleteState {
    fn from((state, response): (RequestedState, Response)) -> CompleteState {
        trace!("ConnectionInvitee: transit state from RequestedState to CompleteState");
        CompleteState {
            bootstrap_did_doc: state.did_doc,
            did_doc: response.connection.did_doc,
            protocols: None,
        }
    }
}

impl From<RespondedState> for CompleteState {
    fn from(state: RespondedState) -> CompleteState {
        trace!("ConnectionInvitee: transit state from RespondedState to CompleteState");
        CompleteState {
            bootstrap_did_doc: state.did_doc,
            did_doc: state.response.connection.did_doc,
            protocols: None,
        }
    }
}

impl TheirDidDoc for CompleteState {
    fn their_did_doc(&self) -> &AriesDidDoc {
        &self.did_doc
    }
}