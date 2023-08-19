use agency_client::agency_client::AgencyClient;
use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use messages::msg_fields::protocols::cred_issuance::CredentialIssuance;
use messages::msg_fields::protocols::notification::Notification;
use messages::AriesMessage;
use std::collections::HashMap;
use std::sync::Arc;

use crate::errors::error::prelude::*;
use crate::handlers::connection::mediated_connection::MediatedConnection;
use crate::handlers::issuance::holder::Holder;
use crate::handlers::util::{matches_opt_thread_id, matches_thread_id};
use crate::protocols::issuance::holder::state_machine::HolderState;

pub async fn holder_update_with_mediator(
    sm: &mut Holder,
    ledger: &Arc<dyn AnoncredsLedgerRead>,
    anoncreds: &Arc<dyn BaseAnonCreds>,
    wallet: &Arc<dyn BaseWallet>,
    agency_client: &AgencyClient,
    connection: &MediatedConnection,
) -> VcxResult<HolderState> {
    trace!("Holder::update_state >>>");
    if sm.is_terminal_state() {
        return Ok(sm.get_state());
    }
    let send_message = connection.send_message_closure(Arc::clone(wallet)).await?;

    let messages = connection.get_messages(agency_client).await?;
    if let Some((uid, msg)) = holder_find_message_to_handle(sm, messages) {
        sm.process_aries_msg(ledger, anoncreds, msg.into(), Some(send_message))
            .await?;
        connection.update_message_status(&uid, agency_client).await?;
    }
    Ok(sm.get_state())
}

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

#[allow(clippy::unwrap_used)]
pub fn holder_find_message_to_handle(
    sm: &Holder,
    messages: HashMap<String, AriesMessage>,
) -> Option<(String, AriesMessage)> {
    for (uid, message) in messages {
        match sm.get_state() {
            HolderState::ProposalSent => {
                if let AriesMessage::CredentialIssuance(CredentialIssuance::OfferCredential(offer)) = &message {
                    if matches_opt_thread_id!(offer, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
            }
            HolderState::RequestSent => match &message {
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
