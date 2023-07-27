pub mod handlers;
pub mod state;

use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::connection::{
        response::{ConnectionSignature, Response, ResponseContent, ResponseDecorators},
        ConnectionData,
    },
    AriesMessage,
};
use uuid::Uuid;

use self::state::{InviterComplete, InviterRequested, InviterResponded};

#[derive(Clone, Debug)]
pub struct InviterConnection<S> {
    state: S,
}

impl InviterConnection<InviterRequested> {
    pub fn new_inviter(self, con_data: ConnectionData) -> Self {
        InviterConnection {
            state: InviterRequested { con_data },
        }
    }
}

impl InviterConnection<InviterRequested> {
    pub fn into_responded(
        self,
        con_sig: ConnectionSignature,
        thread: Thread,
        timing: Timing,
    ) -> (InviterConnection<InviterResponded>, Response) {
        let id = Uuid::new_v4().to_string();
        let content = ResponseContent::new(con_sig);

        let mut decorators = ResponseDecorators::new(thread);
        decorators.timing = Some(timing);

        let response = Response::with_decorators(id, content, decorators);

        let sm = InviterConnection {
            state: InviterResponded {
                con_data: self.state.con_data,
            },
        };

        (sm, response)
    }
}

impl InviterConnection<InviterResponded> {
    pub fn into_complete(self, _: &AriesMessage) -> InviterConnection<InviterComplete> {
        InviterConnection {
            state: InviterComplete {
                con_data: self.state.con_data,
            },
        }
    }
}
