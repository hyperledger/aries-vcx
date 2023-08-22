use agency_client::agency_client::AgencyClient;
use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use messages::msg_fields::protocols::notification::Notification;
use messages::msg_fields::protocols::present_proof::PresentProof;
use messages::AriesMessage;
use std::collections::HashMap;
use std::sync::Arc;

use crate::errors::error::prelude::*;
use crate::handlers::connection::mediated_connection::MediatedConnection;
use crate::handlers::proof_presentation::prover::Prover;
use crate::handlers::util::{matches_opt_thread_id, matches_thread_id};
use crate::protocols::proof_presentation::prover::state_machine::ProverState;

// todo: returns specific type
pub async fn get_proof_request_messages(
    agency_client: &AgencyClient,
    connection: &MediatedConnection,
) -> VcxResult<String> {
    let presentation_requests: Vec<AriesMessage> = connection
        .get_messages(agency_client)
        .await?
        .into_iter()
        .filter_map(|(_, message)| match message {
            AriesMessage::PresentProof(PresentProof::RequestPresentation(_)) => Some(message),
            _ => None,
        })
        .collect();

    Ok(json!(presentation_requests).to_string())
}

pub async fn prover_update_with_mediator(
    sm: &mut Prover,
    agency_client: &AgencyClient,
    connection: &MediatedConnection,
) -> VcxResult<ProverState> {
    trace!("prover_update_with_mediator >>> ");
    if !sm.progressable_by_message() {
        return Ok(sm.get_state());
    }
    let messages = connection.get_messages(agency_client).await?;
    if let Some((uid, msg)) = prover_find_message_to_handle(sm, messages) {
        sm.process_aries_msg(msg.into()).await?;
        connection.update_message_status(&uid, agency_client).await?;
    }
    Ok(sm.get_state())
}

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
