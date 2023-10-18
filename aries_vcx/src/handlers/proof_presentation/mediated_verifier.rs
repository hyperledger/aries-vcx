use std::collections::HashMap;

use messages::{
    msg_fields::protocols::present_proof::{v1::PresentProofV1, PresentProof},
    AriesMessage,
};

use crate::{
    handlers::{
        proof_presentation::verifier::Verifier,
        util::{matches_opt_thread_id, matches_thread_id},
    },
    protocols::proof_presentation::verifier::state_machine::VerifierState,
};

#[allow(clippy::unwrap_used)]
pub fn verifier_find_message_to_handle(
    sm: &Verifier,
    messages: HashMap<String, AriesMessage>,
) -> Option<(String, AriesMessage)> {
    trace!(
        "verifier_find_message_to_handle >>> messages: {:?}",
        messages
    );
    for (uid, message) in messages {
        match sm.get_state() {
            VerifierState::Initial => match &message {
                AriesMessage::PresentProof(PresentProof::V1(
                    PresentProofV1::ProposePresentation(_),
                )) => {
                    return Some((uid, message));
                }
                AriesMessage::PresentProof(PresentProof::V1(
                    PresentProofV1::RequestPresentation(_),
                )) => {
                    return Some((uid, message));
                }
                _ => {}
            },
            VerifierState::PresentationRequestSent => match &message {
                AriesMessage::PresentProof(PresentProof::V1(PresentProofV1::Presentation(
                    presentation,
                ))) => {
                    if matches_thread_id!(presentation, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::PresentProof(PresentProof::V1(
                    PresentProofV1::ProposePresentation(proposal),
                )) => {
                    if matches_opt_thread_id!(proposal, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::ReportProblem(problem_report) => {
                    if matches_opt_thread_id!(problem_report, sm.get_thread_id().unwrap().as_str())
                    {
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
