use std::sync::Arc;

use anoncreds_types::data_types::messages::pres_request::PresentationRequest;
use aries_vcx::{
    handlers::proof_presentation::verifier::Verifier,
    messages::{
        msg_fields::protocols::present_proof::v1::{
            present::PresentationV1, propose::ProposePresentationV1,
        },
        AriesMessage,
    },
    protocols::{
        proof_presentation::verifier::{
            state_machine::VerifierState, verification_status::PresentationVerificationStatus,
        },
        SendClosure,
    },
};
use aries_vcx_anoncreds::anoncreds::anoncreds::Anoncreds;
use aries_vcx_ledger::ledger::indy_vdr_ledger::DefaultIndyLedgerRead;
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;

use super::connection::ServiceConnections;
use crate::{
    error::*,
    http::VcxHttpClient,
    storage::{agent_storage_inmem::AgentStorageInMem, AgentStorage},
};

#[derive(Clone)]
struct VerifierWrapper {
    verifier: Verifier,
    connection_id: String,
}

impl VerifierWrapper {
    pub fn new(verifier: Verifier, connection_id: &str) -> Self {
        Self {
            verifier,
            connection_id: connection_id.to_string(),
        }
    }
}

pub struct ServiceVerifier<T> {
    ledger_read: Arc<DefaultIndyLedgerRead>,
    anoncreds: Anoncreds,
    wallet: Arc<T>,
    verifiers: AgentStorageInMem<VerifierWrapper>,
    service_connections: Arc<ServiceConnections<T>>,
}

impl<T: BaseWallet> ServiceVerifier<T> {
    pub fn new(
        ledger_read: Arc<DefaultIndyLedgerRead>,
        anoncreds: Anoncreds,
        wallet: Arc<T>,
        service_connections: Arc<ServiceConnections<T>>,
    ) -> Self {
        Self {
            service_connections,
            verifiers: AgentStorageInMem::new("verifiers"),
            ledger_read,
            anoncreds,
            wallet,
        }
    }

    pub async fn send_proof_request(
        &self,
        connection_id: &str,
        request: PresentationRequest,
        proposal: Option<ProposePresentationV1>,
    ) -> AgentResult<String> {
        let connection = self.service_connections.get_by_id(connection_id)?;
        let mut verifier = if let Some(proposal) = proposal {
            Verifier::create_from_proposal("", &proposal)?
        } else {
            Verifier::create_from_request("".to_string(), &request)?
        };

        let send_closure: SendClosure = Box::new(|msg: AriesMessage| {
            Box::pin(async move {
                connection
                    .send_message(self.wallet.as_ref(), &msg, &VcxHttpClient)
                    .await
            })
        });

        let message = verifier.mark_presentation_request_sent()?;
        send_closure(message.into()).await?;
        self.verifiers.insert(
            &verifier.get_thread_id()?,
            VerifierWrapper::new(verifier, connection_id),
        )
    }

    pub fn get_presentation_status(
        &self,
        thread_id: &str,
    ) -> AgentResult<PresentationVerificationStatus> {
        let VerifierWrapper { verifier, .. } = self.verifiers.get(thread_id)?;
        Ok(verifier.get_verification_status())
    }

    pub async fn verify_presentation(
        &self,
        thread_id: &str,
        presentation: PresentationV1,
    ) -> AgentResult<()> {
        let VerifierWrapper {
            mut verifier,
            connection_id,
        } = self.verifiers.get(thread_id)?;
        let connection = self.service_connections.get_by_id(&connection_id)?;

        let send_closure: SendClosure = Box::new(|msg: AriesMessage| {
            Box::pin(async move {
                connection
                    .send_message(self.wallet.as_ref(), &msg, &VcxHttpClient)
                    .await
            })
        });

        let message = verifier
            .verify_presentation(self.ledger_read.as_ref(), &self.anoncreds, presentation)
            .await?;
        send_closure(message).await?;
        self.verifiers
            .insert(thread_id, VerifierWrapper::new(verifier, &connection_id))?;
        Ok(())
    }

    pub fn get_state(&self, thread_id: &str) -> AgentResult<VerifierState> {
        let VerifierWrapper { verifier, .. } = self.verifiers.get(thread_id)?;
        Ok(verifier.get_state())
    }

    pub fn exists_by_id(&self, thread_id: &str) -> bool {
        self.verifiers.contains_key(thread_id)
    }
}
