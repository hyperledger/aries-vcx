use std::collections::HashMap;

use messages::msg_fields::protocols::cred_issuance::CredentialIssuance;
use messages::msg_fields::protocols::notification::Notification;
use messages::AriesMessage;

use crate::handlers::issuance::holder::Holder;
use crate::handlers::util::{matches_opt_thread_id, matches_thread_id};
use crate::protocols::issuance::holder::state_machine::HolderState;

#[allow(clippy::unwrap_used)]
pub fn holder_find_message_to_handle(
    sm: &Holder,
    messages: HashMap<String, AriesMessage>,
) -> Option<(String, AriesMessage)> {
    trace!("holder_find_message_to_handle >>>");
    for (uid, message) in messages {
        match sm.get_state() {
            HolderState::ProposalSent => {
                if let AriesMessage::CredentialIssuance(CredentialIssuance::OfferCredential(offer)) = &message {
                    if matches_opt_thread_id!(offer, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
            }
            HolderState::RequestSet => match &message {
                AriesMessage::CredentialIssuance(CredentialIssuance::IssueCredential(credential)) => {
                    if matches_thread_id!(credential, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::CredentialIssuance(CredentialIssuance::ProblemReport(problem_report)) => {
                    if matches_opt_thread_id!(problem_report, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::ReportProblem(problem_report) => {
                    if matches_opt_thread_id!(problem_report, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::Notification(Notification::ProblemReport(msg)) => {
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
