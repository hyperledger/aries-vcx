use agency_client::agency_client::AgencyClient;
use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use messages::msg_fields::protocols::present_proof::PresentProof;
use messages::AriesMessage;
use std::collections::HashMap;
use std::sync::Arc;

use crate::errors::error::VcxResult;
use crate::handlers::connection::mediated_connection::MediatedConnection;
use crate::handlers::proof_presentation::verifier::Verifier;
use crate::handlers::util::{matches_opt_thread_id, matches_thread_id};
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
    if let Some((uid, msg)) = verifier_find_message_to_handle(sm, messages) {
        sm.process_aries_msg(ledger, anoncreds, msg.into(), Some(send_message))
            .await?;
        connection.update_message_status(&uid, agency_client).await?;
    }
    Ok(sm.get_state())
}

#[allow(clippy::unwrap_used)]
pub fn verifier_find_message_to_handle(
    sm: &Verifier,
    messages: HashMap<String, AriesMessage>,
) -> Option<(String, AriesMessage)> {
    trace!("VerifierSM::find_message_to_handle >>> messages: {:?}", messages);
    for (uid, message) in messages {
        match sm.get_state() {
            VerifierState::Initial => match &message {
                AriesMessage::PresentProof(PresentProof::ProposePresentation(_)) => {
                    return Some((uid, message));
                }
                AriesMessage::PresentProof(PresentProof::RequestPresentation(_)) => {
                    return Some((uid, message));
                }
                _ => {}
            },
            VerifierState::PresentationRequestSent => match &message {
                AriesMessage::PresentProof(PresentProof::Presentation(presentation)) => {
                    if matches_thread_id!(presentation, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::PresentProof(PresentProof::ProposePresentation(proposal)) => {
                    if matches_opt_thread_id!(proposal, sm.get_thread_id().unwrap().as_str()) {
                        return Some((uid, message));
                    }
                }
                AriesMessage::ReportProblem(problem_report) => {
                    if matches_opt_thread_id!(problem_report, sm.get_thread_id().unwrap().as_str()) {
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
