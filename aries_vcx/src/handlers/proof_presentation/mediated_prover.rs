use agency_client::agency_client::AgencyClient;
use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use messages::msg_fields::protocols::present_proof::PresentProof;
use messages::AriesMessage;
use std::sync::Arc;

use crate::errors::error::prelude::*;
use crate::handlers::connection::mediated_connection::MediatedConnection;
use crate::handlers::proof_presentation::prover::Prover;
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
    ledger: &Arc<dyn AnoncredsLedgerRead>,
    anoncreds: &Arc<dyn BaseAnonCreds>,
    wallet: &Arc<dyn BaseWallet>,
    agency_client: &AgencyClient,
    connection: &MediatedConnection,
) -> VcxResult<ProverState> {
    trace!("Prover::update_state >>> ");
    if !sm.progressable_by_message() {
        return Ok(sm.get_state());
    }
    let send_message = connection.send_message_closure(Arc::clone(wallet)).await?;

    let messages = connection.get_messages(agency_client).await?;
    if let Some((uid, msg)) = sm.find_message_to_handle(messages) {
        sm.step(ledger, anoncreds, msg.into(), Some(send_message)).await?;
        connection.update_message_status(&uid, agency_client).await?;
    }
    Ok(sm.get_state())
}
