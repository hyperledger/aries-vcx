pub mod handlers;
pub mod state;

use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::{
        connection::{
            request::{Request, RequestContent, RequestDecorators},
            ConnectionData,
        },
        notification::ack::{Ack, AckContent, AckDecorators, AckStatus},
    },
};
use uuid::Uuid;

use self::state::{InviteeComplete, InviteeRequested, InviteeResponded};

pub struct InviteeConnection<S> {
    state: S,
}

impl InviteeConnection<InviteeRequested> {
    pub fn new_invitee(
        recipient_keys: Vec<String>,
        label: String,
        con_data: ConnectionData,
        thread: Thread,
        timing: Timing,
    ) -> (Self, Request) {
        let id = Uuid::new_v4().to_string();
        let content = RequestContent::new(label, con_data);

        let decorators = RequestDecorators {
            thread: Some(thread),
            timing: Some(timing),
        };

        let request = Request::with_decorators(id, content, decorators);

        let sm = Self {
            state: InviteeRequested { recipient_keys },
        };

        (sm, request)
    }

    pub fn into_responded(self, con_data: ConnectionData) -> InviteeConnection<InviteeResponded> {
        InviteeConnection {
            state: InviteeResponded { con_data },
        }
    }
}

impl InviteeConnection<InviteeResponded> {
    pub fn into_complete(self) -> InviteeConnection<InviteeComplete> {
        InviteeConnection {
            state: InviteeComplete {
                con_data: self.state.con_data,
            },
        }
    }

    pub fn into_complete_with_ack(self, thread: Thread, timing: Timing) -> (InviteeConnection<InviteeComplete>, Ack) {
        let sm = InviteeConnection {
            state: InviteeComplete {
                con_data: self.state.con_data,
            },
        };

        let id = Uuid::new_v4().to_string();
        let content = AckContent::new(AckStatus::Ok);

        let decorators = AckDecorators {
            thread,
            timing: Some(timing),
        };

        let ack = Ack::with_decorators(id, content, decorators);

        (sm, ack)
    }
}
