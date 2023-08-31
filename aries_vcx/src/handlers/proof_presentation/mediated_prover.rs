use std::collections::HashMap;

use messages::msg_fields::protocols::notification::Notification;
use messages::msg_fields::protocols::present_proof::PresentProof;
use messages::AriesMessage;

use crate::handlers::proof_presentation::prover::Prover;
use crate::handlers::util::{matches_opt_thread_id, matches_thread_id};
use crate::protocols::proof_presentation::prover::state_machine::ProverState;

#[allow(clippy::unwrap_used)]
pub fn prover_find_message_to_handle(
    sm: &Prover,
    messages: HashMap<String, AriesMessage>,
) -> Option<(String, AriesMessage)> {
    trace!("prover_find_message_to_handle >>> messages: {:?}", messages);
    for (uid, message) in messages {
        match sm.get_state() {
            ProverState::PresentationProposalSent => match &message {
                AriesMessage::ReportProblem(msg) => {
                    if matches_opt_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::Notification(Notification::ProblemReport(msg)) => {
                    if matches_opt_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::PresentProof(PresentProof::RequestPresentation(msg)) => {
                    if matches_opt_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                _ => {}
            },
            ProverState::PresentationSent => match &message {
                AriesMessage::Notification(Notification::Ack(msg)) => {
                    if matches_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::PresentProof(PresentProof::Ack(msg)) => {
                    if matches_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::ReportProblem(msg) => {
                    if matches_opt_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::Notification(Notification::ProblemReport(msg)) => {
                    if matches_opt_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::PresentProof(PresentProof::ProblemReport(msg)) => {
                    if matches_opt_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                _ => {}
            },
            _ => {}
        };
    }
    None
}
