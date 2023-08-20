use std::collections::HashMap;

use messages::msg_fields::protocols::cred_issuance::CredentialIssuance;
use messages::msg_fields::protocols::notification::Notification;
use messages::AriesMessage;

use crate::handlers::issuance::issuer::Issuer;
use crate::handlers::util::{matches_opt_thread_id, matches_thread_id};
use crate::protocols::issuance::issuer::state_machine::IssuerState;

#[allow(clippy::unwrap_used)]
pub fn issuer_find_message_to_handle(
    sm: &Issuer,
    messages: HashMap<String, AriesMessage>,
) -> Option<(String, AriesMessage)> {
    trace!(
        "issuer_find_messages_to_handle >>> messages: {:?}, state: {:?}",
        messages,
        sm
    );

    for (uid, message) in messages {
        match sm.get_state() {
            IssuerState::Initial => {
                if let AriesMessage::CredentialIssuance(CredentialIssuance::ProposeCredential(_)) = &message {
                    return Some((uid, message));
                }
            }
            IssuerState::OfferSet => match &message {
                AriesMessage::CredentialIssuance(CredentialIssuance::RequestCredential(msg)) => {
                    if matches_opt_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::CredentialIssuance(CredentialIssuance::ProposeCredential(msg)) => {
                    if matches_opt_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::ReportProblem(msg) => {
                    if matches_opt_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                _ => {}
            },
            IssuerState::CredentialSet => match &message {
                AriesMessage::CredentialIssuance(CredentialIssuance::Ack(msg)) => {
                    if matches_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::Notification(Notification::Ack(msg)) => {
                    if matches_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::ReportProblem(msg) => {
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
