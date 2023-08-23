use messages::msg_fields::protocols::present_proof::PresentProof;
use messages::AriesMessage;
use std::collections::HashMap;
use crate::handlers::proof_presentation::verifier::Verifier;
use crate::handlers::util::{matches_opt_thread_id, matches_thread_id};
use crate::protocols::proof_presentation::verifier::state_machine::VerifierState;

#[allow(clippy::unwrap_used)]
pub fn verifier_find_message_to_handle(
    sm: &Verifier,
    messages: HashMap<String, AriesMessage>,
) -> Option<(String, AriesMessage)> {
    trace!("verifier_find_message_to_handle >>> messages: {:?}", messages);
    for (uid, message) in messages {
        match sm.get_state() {
            VerifierState::Initial => match &message {
                AriesMessage::PresentProof(PresentProof::ProposePresentation(_)) => {
                    return Some((uid, message));
                }
                AriesMessage::PresentProof(PresentProof::RequestPresentation(_)) => {
                    return Some((uid, message));
                }
                _ => {}
            },
            VerifierState::PresentationRequestSent => match &message {
                AriesMessage::PresentProof(PresentProof::Presentation(presentation)) => {
                    if matches_thread_id!(presentation, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::PresentProof(PresentProof::ProposePresentation(proposal)) => {
                    if matches_opt_thread_id!(proposal, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::ReportProblem(problem_report) => {
                    if matches_opt_thread_id!(problem_report, sm.get_thread_id().unwrap().as_str()) {
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
