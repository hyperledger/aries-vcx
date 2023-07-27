pub mod handlers;
pub mod state;

use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::msg_fields::protocols::{
    connection::{request::RequestContent, ConnectionData},
    notification::ack::{AckContent, AckStatus},
};

use self::state::{BootstrapInfo, InviteeComplete, InviteeRequested};

#[derive(Clone, Debug)]
pub struct InviteeConnection<S> {
    did: String,
    verkey: String,
    state: S,
}

impl InviteeConnection<InviteeRequested> {
    pub fn new_invitee(
        did: String,
        verkey: String,
        label: String,
        bootstrap_info: BootstrapInfo,
        con_data: ConnectionData,
    ) -> (Self, RequestContent) {
        let content = RequestContent::new(label, con_data);

        let sm = Self {
            state: InviteeRequested { bootstrap_info },
            did,
            verkey,
        };

        (sm, content)
    }

    pub fn into_complete(self, did_doc: AriesDidDoc) -> (InviteeConnection<InviteeComplete>, AckContent) {
        let sm = InviteeConnection {
            did: self.did,
            verkey: self.verkey,
            state: InviteeComplete {
                did_doc,
                bootstrap_info: self.state.bootstrap_info,
            },
        };

        let content = AckContent::new(AckStatus::Ok);
        (sm, content)
    }
}
