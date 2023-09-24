use std::collections::HashMap;

use messages::{
    msg_fields::protocols::{
        cred_issuance::{v1::CredentialIssuanceV1, CredentialIssuance},
        notification::Notification,
    },
    AriesMessage,
};

use crate::{
    handlers::{
        issuance::issuer::Issuer,
        util::{matches_opt_thread_id, matches_thread_id},
    },
    protocols::issuance::issuer::state_machine::IssuerState,
};

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
                if let AriesMessage::CredentialIssuance(CredentialIssuance::V1(
                    CredentialIssuanceV1::ProposeCredential(_),
                )) = &message
                {
                    info!(
                        "In state IssuerState::OfferSet, found matching message ProposeCredential"
                    );
                    return Some((uid, message));
                }
            }
            IssuerState::OfferSet => match &message {
                AriesMessage::CredentialIssuance(CredentialIssuance::V1(
                    CredentialIssuanceV1::RequestCredential(msg),
                )) => {
                    info!(
                        "In state IssuerState::OfferSet, found potentially matching message \
                         RequestCredential"
                    );
                    warn!("Matching for {}", sm.get_thread_id().unwrap().as_str()); // todo: the state machine has "test" thid, and doesnt match msg
                    warn!("Msg: {msg:?}");
                    if matches_opt_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::CredentialIssuance(CredentialIssuance::V1(
                    CredentialIssuanceV1::ProposeCredential(msg),
                )) => {
                    info!(
                        "In state IssuerState::OfferSet, found potentially matching message \
                         ProposeCredential"
                    );
                    if matches_opt_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::ReportProblem(msg) => {
                    info!("In state IssuerState::OfferSet, found matching message ReportProblem");
                    if matches_opt_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                _ => {}
            },
            IssuerState::CredentialSet => match &message {
                AriesMessage::CredentialIssuance(CredentialIssuance::V1(
                    CredentialIssuanceV1::Ack(msg),
                )) => {
                    info!(
                        "In state IssuerState::CredentialSet, found matching message \
                         CredentialIssuance::Ack"
                    );
                    if matches_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::Notification(Notification::Ack(msg)) => {
                    info!(
                        "In state IssuerState::CredentialSet, found matching message \
                         Notification::Ack"
                    );
                    if matches_thread_id!(msg, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::ReportProblem(msg) => {
                    info!(
                        "In state IssuerState::CredentialSet, found matching message ReportProblem"
                    );
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
