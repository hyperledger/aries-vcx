use agency_client::agency_client::AgencyClient;
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::errors::error::VcxResult;
use aries_vcx::handlers::connection::mediated_connection::MediatedConnection;
use aries_vcx::handlers::issuance::holder::Holder;
use aries_vcx::handlers::issuance::issuer::Issuer;
use aries_vcx::handlers::issuance::{mediated_holder, mediated_issuer};
use aries_vcx::handlers::proof_presentation::prover::Prover;
use aries_vcx::handlers::proof_presentation::verifier::Verifier;
use aries_vcx::handlers::proof_presentation::{mediated_prover, mediated_verifier};
use aries_vcx::protocols::issuance::holder::state_machine::HolderState;
use aries_vcx::protocols::issuance::issuer::state_machine::IssuerState;
use aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;
use aries_vcx::protocols::proof_presentation::verifier::state_machine::VerifierState;
use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use messages::msg_fields::protocols::cred_issuance::propose_credential::ProposeCredential;
use messages::msg_fields::protocols::cred_issuance::CredentialIssuance;
use messages::msg_fields::protocols::present_proof::PresentProof;
use messages::AriesMessage;
use std::sync::Arc;

#[cfg(test)]
pub mod test_utils {
    use agency_client::api::downloaded_message::DownloadedMessage;
    use aries_vcx::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};
    use messages::msg_fields::protocols::connection::Connection;
    use messages::msg_fields::protocols::cred_issuance::CredentialIssuance;
    use messages::msg_fields::protocols::present_proof::PresentProof;
    use messages::AriesMessage;

    #[derive(Debug)]
    pub struct VcxAgencyMessage {
        pub uid: String,
        pub decrypted_msg: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum PayloadKinds {
        CredOffer,
        CredReq,
        Cred,
        Proof,
        ProofRequest,
        ConnRequest,
        Other(String),
    }

    fn determine_message_type(a2a_message: AriesMessage) -> PayloadKinds {
        debug!("determine_message_type >>> a2a_message: {:?}", a2a_message);
        match a2a_message.clone() {
            AriesMessage::PresentProof(PresentProof::RequestPresentation(_)) => PayloadKinds::ProofRequest,
            AriesMessage::CredentialIssuance(CredentialIssuance::OfferCredential(_)) => PayloadKinds::CredOffer,
            AriesMessage::CredentialIssuance(CredentialIssuance::IssueCredential(_)) => PayloadKinds::Cred,
            AriesMessage::PresentProof(PresentProof::Presentation(_)) => PayloadKinds::Proof,
            AriesMessage::Connection(Connection::Request(_)) => PayloadKinds::ConnRequest,
            _msg => PayloadKinds::Other(String::from("aries")),
        }
    }

    fn str_message_to_a2a_message(message: &str) -> VcxResult<AriesMessage> {
        Ok(serde_json::from_str(message).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Cannot deserialize A2A message: {}", err),
            )
        })?)
    }

    fn str_message_to_payload_type(message: &str) -> VcxResult<PayloadKinds> {
        let a2a_message = str_message_to_a2a_message(message)?;
        Ok(determine_message_type(a2a_message))
    }

    pub async fn filter_messages(
        messages: Vec<DownloadedMessage>,
        filter_msg_type: PayloadKinds,
    ) -> Option<VcxAgencyMessage> {
        for message in messages.into_iter() {
            let decrypted_msg = &message.decrypted_msg;
            let msg_type = str_message_to_payload_type(decrypted_msg).unwrap();
            if filter_msg_type == msg_type {
                return Some(VcxAgencyMessage {
                    uid: message.uid,
                    decrypted_msg: decrypted_msg.to_string(),
                });
            }
        }
        None
    }
}

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
    if let Some((uid, msg)) = mediated_prover::prover_find_message_to_handle(sm, messages) {
        sm.process_aries_msg(msg.into()).await?;
        connection.update_message_status(&uid, agency_client).await?;
    }
    Ok(sm.get_state())
}

pub async fn verifier_update_with_mediator(
    sm: &mut Verifier,
    wallet: &Arc<dyn BaseWallet>,
    ledger: &Arc<dyn AnoncredsLedgerRead>,
    anoncreds: &Arc<dyn BaseAnonCreds>,
    agency_client: &AgencyClient,
    connection: &MediatedConnection,
) -> VcxResult<VerifierState> {
    trace!("verifier_update_with_mediator >>> ");
    if !sm.progressable_by_message() {
        return Ok(sm.get_state());
    }
    let send_message = connection.send_message_closure(Arc::clone(wallet)).await?;

    let messages = connection.get_messages(agency_client).await?;
    if let Some((uid, msg)) = mediated_verifier::verifier_find_message_to_handle(sm, messages) {
        if let Some(message) = sm.process_aries_msg(ledger, anoncreds, msg.into()).await? {
            send_message(message).await?;
        }
        connection.update_message_status(&uid, agency_client).await?;
    }
    Ok(sm.get_state())
}

pub async fn holder_update_with_mediator(
    sm: &mut Holder,
    ledger: &Arc<dyn AnoncredsLedgerRead>,
    anoncreds: &Arc<dyn BaseAnonCreds>,
    wallet: &Arc<dyn BaseWallet>,
    agency_client: &AgencyClient,
    connection: &MediatedConnection,
) -> VcxResult<HolderState> {
    trace!("holder_update_with_mediator >>>");
    if sm.is_terminal_state() {
        return Ok(sm.get_state());
    }
    let send_message = connection.send_message_closure(Arc::clone(wallet)).await?;

    let messages = connection.get_messages(agency_client).await?;
    if let Some((uid, msg)) = mediated_holder::holder_find_message_to_handle(sm, messages) {
        sm.process_aries_msg(ledger, anoncreds, msg.clone()).await?;
        connection.update_message_status(&uid, agency_client).await?;
        sm.try_reply(send_message, Some(msg)).await?;
    }
    Ok(sm.get_state())
}

// todo: returns specific type
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

pub async fn issuer_update_with_mediator(
    sm: &mut Issuer,
    agency_client: &AgencyClient,
    connection: &MediatedConnection,
) -> VcxResult<IssuerState> {
    trace!("issuer_update_with_mediator >>>");
    let messages = connection.get_messages(agency_client).await?;
    if let Some((uid, msg)) = mediated_issuer::issuer_find_message_to_handle(sm, messages) {
        trace!("Issuer::update_state >>> found msg to handle; uid: {uid}, msg: {msg:?}");
        sm.process_aries_msg(msg.into()).await?;
        connection.update_message_status(&uid, agency_client).await?;
    } else {
        trace!("Issuer::update_state >>> found no msg to handle");
    }
    Ok(sm.get_state())
}

// todo: returns specific type
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
