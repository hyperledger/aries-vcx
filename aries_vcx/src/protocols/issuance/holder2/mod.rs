use messages::msg_fields::protocols::{
    cred_issuance::{propose_credential::ProposeCredential, request_credential::RequestCredential},
    notification::ack::Ack,
    report_problem::ProblemReport,
};

use self::states::{
    failed::Failed, finished::Finished, proposal_prepared::ProposalPrepared, request_prepared::RequestPrepared,
};

pub mod states;
pub mod transitions;

#[derive(Debug)]
pub struct Holder<S> {
    thread_id: String,
    state: S,
}

impl<S> Holder<S> {
    pub fn from_parts(thread_id: String, state: S) -> Self {
        Self { thread_id, state }
    }

    pub fn into_parts(self) -> (String, S) {
        let Self { thread_id, state } = self;
        (thread_id, state)
    }

    pub fn thread_id(&self) -> &str {
        &self.thread_id
    }
}

impl Holder<ProposalPrepared> {
    pub fn get_proposal_message(&self) -> ProposeCredential {
        self.state.proposal_message.clone()
    }
}

impl Holder<RequestPrepared> {
    pub fn get_request_message(&self) -> RequestCredential {
        todo!()
    }
}

impl Holder<Finished> {
    pub fn get_ack_message(&self) -> Ack {
        todo!()
    }
}

impl Holder<Failed> {
    pub fn get_problem_report_message(&self) -> ProblemReport {
        todo!()
    }
}
