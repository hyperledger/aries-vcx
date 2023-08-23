mod helpers;

pub mod generic;
pub mod requester;
pub mod responder;

pub use helpers::generate_keypair;
use messages::{
    decorators::thread::Thread,
    msg_fields::protocols::did_exchange::problem_report::{
        ProblemCode, ProblemReport, ProblemReportContent, ProblemReportDecorators,
    },
};
use uuid::Uuid;

use std::marker::PhantomData;

use did_doc_sov::DidDocumentSov;

use super::{
    states::{abandoned::Abandoned, traits::ThreadId},
    transition::transition_result::TransitionResult,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DidExchange<I, S> {
    state: S,
    initiation_type: PhantomData<I>,
    our_did_document: DidDocumentSov,
    their_did_document: DidDocumentSov,
}

impl<I, S> DidExchange<I, S> {
    pub fn our_did_doc(&self) -> &DidDocumentSov {
        &self.our_did_document
    }

    pub fn their_did_doc(&self) -> &DidDocumentSov {
        &self.their_did_document
    }
}

impl<I, S: ThreadId> DidExchange<I, S> {
    pub fn get_thread_id(&self) -> &str {
        self.state.thread_id()
    }

    pub fn fail(
        self,
        reason: String,
        problem_code: Option<ProblemCode>,
    ) -> TransitionResult<DidExchange<I, Abandoned>, ProblemReport>
    where
        S: ThreadId,
    {
        let problem_report = {
            let id = Uuid::new_v4().to_string();
            let content = ProblemReportContent {
                problem_code,
                explain: Some(reason.clone()),
            };
            let decorators = ProblemReportDecorators {
                // TODO: Set thid of the conversation
                thread: Thread::new(self.state.thread_id().to_string()),
                localization: None,
                timing: None,
            };
            ProblemReport::with_decorators(id, content, decorators)
        };
        TransitionResult {
            state: DidExchange {
                state: Abandoned {
                    reason,
                    request_id: self.state.thread_id().to_string(),
                },
                initiation_type: PhantomData,
                our_did_document: self.our_did_document,
                their_did_document: self.their_did_document,
            },
            output: problem_report,
        }
    }
}

impl<I, S> DidExchange<I, S> {
    pub fn from_parts(state: S, their_did_document: DidDocumentSov, our_did_document: DidDocumentSov) -> Self {
        Self {
            state,
            initiation_type: PhantomData,
            our_did_document,
            their_did_document,
        }
    }
}
