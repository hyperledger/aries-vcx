use crate::errors::error::VcxResult;
use crate::handlers::connection::mediated_connection::MediatedConnection;
use crate::handlers::issuance::issuer::Issuer;
use crate::handlers::util::{matches_opt_thread_id, matches_thread_id};
use crate::protocols::issuance::issuer::state_machine::{IssuerFullState, IssuerSM, IssuerState};
use agency_client::agency_client::AgencyClient;
use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use messages::msg_fields::protocols::cred_issuance::propose_credential::ProposeCredential;
use messages::msg_fields::protocols::cred_issuance::CredentialIssuance;
use messages::msg_fields::protocols::notification::Notification;
use messages::AriesMessage;
use std::collections::HashMap;
use std::sync::Arc;

// todo: remove unused args
pub async fn issuer_update_with_mediator(
    sm: &mut Issuer,
    _wallet: &Arc<dyn BaseWallet>,
    _anoncreds: &Arc<dyn BaseAnonCreds>,
    agency_client: &AgencyClient,
    connection: &MediatedConnection,
) -> VcxResult<IssuerState> {
    trace!("Issuer::update_state >>>");
    let messages = connection.get_messages(agency_client).await?;
    if let Some((uid, msg)) = issuer_find_messages_to_handle(sm, messages) {
        trace!("Issuer::update_state >>> found msg to handle; uid: {uid}, msg: {msg:?}");
        sm.process_aries_msg(msg.into()).await?;
        connection.update_message_status(&uid, agency_client).await?;
    } else {
        trace!("Issuer::update_state >>> found no msg to handle");
    }
    Ok(sm.get_state())
}

pub fn issuer_find_messages_to_handle(
    sm: &Issuer,
    messages: HashMap<String, AriesMessage>,
) -> Option<(String, AriesMessage)> {
    trace!(
        "IssuerSM::find_message_to_handle >>> messages: {:?}, state: {:?}",
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
            IssuerState::OfferSent => match &message {
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
            IssuerState::CredentialSent => match &message {
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

pub async fn get_credential_proposal_messages(
    agency_client: &AgencyClient,
    connection: &MediatedConnection,
) -> VcxResult<Vec<(String, ProposeCredential)>> {
    let credential_proposals: Vec<(String, ProposeCredential)> = connection
        .get_messages(agency_client)
        .await?
        .into_iter()
        .filter_map(|(uid, message)| match message {
            AriesMessage::CredentialIssuance(CredentialIssuance::ProposeCredential(proposal)) => Some((uid, proposal)),
            _ => None,
        })
        .collect();

    Ok(credential_proposals)
}
