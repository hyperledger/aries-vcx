use agency_client::agency_client::AgencyClient;
use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use std::sync::Arc;

use crate::errors::error::VcxResult;
use crate::handlers::connection::mediated_connection::MediatedConnection;
use crate::handlers::proof_presentation::verifier::Verifier;
use crate::protocols::proof_presentation::verifier::state_machine::VerifierState;

pub async fn verifier_update_with_mediator(
    sm: &mut Verifier,
    wallet: &Arc<dyn BaseWallet>,
    ledger: &Arc<dyn AnoncredsLedgerRead>,
    anoncreds: &Arc<dyn BaseAnonCreds>,
    agency_client: &AgencyClient,
    connection: &MediatedConnection,
) -> VcxResult<VerifierState> {
    trace!("Verifier::update_state >>> ");
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
