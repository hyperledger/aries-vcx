use std::collections::HashMap;
use agency_client::agency_client::AgencyClient;
use messages::AriesMessage;
use messages::msg_fields::protocols::cred_issuance::CredentialIssuance;
use messages::msg_fields::protocols::notification::Notification;

use crate::errors::error::prelude::*;
use crate::handlers::connection::mediated_connection::MediatedConnection;
use crate::handlers::util::{matches_opt_thread_id, matches_thread_id};
use crate::protocols::issuance::holder::state_machine::{HolderFullState, HolderSM};

pub async fn get_credential_offer_messages(
    agency_client: &AgencyClient,
    connection: &MediatedConnection,
) -> VcxResult<String> {
    let credential_offers: Vec<AriesMessage> = connection
        .get_messages(agency_client)
        .await?
        .into_iter()
        .filter_map(|(_, a2a_message)| match a2a_message {
            AriesMessage::CredentialIssuance(CredentialIssuance::OfferCredential(_)) => Some(a2a_message),
            _ => None,
        })
        .collect();

    Ok(json!(credential_offers).to_string())
}

pub fn holder_find_message_to_handle(sm: &HolderSM, messages: HashMap<String, AriesMessage>) -> Option<(String, AriesMessage)> {
    trace!(
            "Holder::find_message_to_handle >>> messages: {:?}, state: {:?}",
            messages,
            sm.state
        );
    for (uid, message) in messages {
        match sm.state {
            HolderFullState::ProposalSent(_) => {
                if let AriesMessage::CredentialIssuance(CredentialIssuance::OfferCredential(offer)) = &message {
                    if matches_opt_thread_id!(offer, sm.thread_id.as_str()) {
                        return Some((uid, message));
                    }
                }
            }
            HolderFullState::RequestSent(_) => match &message {
                AriesMessage::CredentialIssuance(CredentialIssuance::IssueCredential(credential)) => {
                    if matches_thread_id!(credential, sm.thread_id.as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::CredentialIssuance(CredentialIssuance::ProblemReport(problem_report)) => {
                    if matches_opt_thread_id!(problem_report, sm.thread_id.as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::ReportProblem(problem_report) => {
                    if matches_opt_thread_id!(problem_report, sm.thread_id.as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::Notification(Notification::ProblemReport(msg)) => {
                    if matches_opt_thread_id!(msg, sm.thread_id.as_str()) {
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
