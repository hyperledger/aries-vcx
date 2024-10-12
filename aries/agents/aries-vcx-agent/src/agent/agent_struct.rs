use std::sync::Arc;

use aries_vcx_anoncreds::anoncreds::anoncreds::Anoncreds;
use aries_vcx_ledger::ledger::indy_vdr_ledger::{DefaultIndyLedgerRead, DefaultIndyLedgerWrite};
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;

use crate::handlers::{
    connection::ServiceConnections, credential_definition::ServiceCredentialDefinitions,
    did_exchange::DidcommHandlerDidExchange, holder::ServiceCredentialsHolder,
    issuer::ServiceCredentialsIssuer, out_of_band::ServiceOutOfBand, prover::ServiceProver,
    revocation_registry::ServiceRevocationRegistries, schema::ServiceSchemas,
    verifier::ServiceVerifier,
};

pub struct Agent<W> {
    pub(super) issuer_did: String,
    pub(super) ledger_read: Arc<DefaultIndyLedgerRead>,
    pub(super) ledger_write: Arc<DefaultIndyLedgerWrite>,
    pub(super) anoncreds: Anoncreds,
    pub(super) wallet: Arc<W>,
    pub(super) connections: Arc<ServiceConnections<W>>,
    pub(super) schemas: Arc<ServiceSchemas<W>>,
    pub(super) cred_defs: Arc<ServiceCredentialDefinitions<W>>,
    pub(super) rev_regs: Arc<ServiceRevocationRegistries<W>>,
    pub(super) holder: Arc<ServiceCredentialsHolder<W>>,
    pub(super) issuer: Arc<ServiceCredentialsIssuer<W>>,
    pub(super) verifier: Arc<ServiceVerifier<W>>,
    pub(super) prover: Arc<ServiceProver<W>>,
    pub(super) out_of_band: Arc<ServiceOutOfBand<W>>,
    pub(super) did_exchange: Arc<DidcommHandlerDidExchange<W>>,
}

// Note: We do this manually, otherwise compiler is requesting us to implement Clone for generic
// type W,       which is not in fact needed - W is wrapped in Arc. Underlying W has no reason to be
// cloned.
impl<W> Clone for Agent<W> {
    fn clone(&self) -> Self {
        Self {
            issuer_did: self.issuer_did.clone(),
            ledger_read: self.ledger_read.clone(),
            ledger_write: self.ledger_write.clone(),
            anoncreds: self.anoncreds,
            wallet: self.wallet.clone(),
            connections: self.connections.clone(),
            schemas: self.schemas.clone(),
            cred_defs: self.cred_defs.clone(),
            rev_regs: self.rev_regs.clone(),
            holder: self.holder.clone(),
            issuer: self.issuer.clone(),
            verifier: self.verifier.clone(),
            prover: self.prover.clone(),
            out_of_band: self.out_of_band.clone(),
            did_exchange: self.did_exchange.clone(),
        }
    }
}

impl<T: BaseWallet> Agent<T> {
    pub fn ledger_read(&self) -> &DefaultIndyLedgerRead {
        &self.ledger_read
    }

    pub fn ledger_write(&self) -> &DefaultIndyLedgerWrite {
        &self.ledger_write
    }

    pub fn anoncreds(&self) -> &Anoncreds {
        &self.anoncreds
    }

    pub fn wallet(&self) -> Arc<T> {
        self.wallet.clone()
    }

    pub fn issuer_did(&self) -> String {
        self.issuer_did.clone()
    }

    pub fn connections(&self) -> Arc<ServiceConnections<T>> {
        self.connections.clone()
    }

    pub fn out_of_band(&self) -> Arc<ServiceOutOfBand<T>> {
        self.out_of_band.clone()
    }

    pub fn did_exchange(&self) -> Arc<DidcommHandlerDidExchange<T>> {
        self.did_exchange.clone()
    }

    pub fn schemas(&self) -> Arc<ServiceSchemas<T>> {
        self.schemas.clone()
    }

    pub fn cred_defs(&self) -> Arc<ServiceCredentialDefinitions<T>> {
        self.cred_defs.clone()
    }

    pub fn rev_regs(&self) -> Arc<ServiceRevocationRegistries<T>> {
        self.rev_regs.clone()
    }

    pub fn issuer(&self) -> Arc<ServiceCredentialsIssuer<T>> {
        self.issuer.clone()
    }

    pub fn holder(&self) -> Arc<ServiceCredentialsHolder<T>> {
        self.holder.clone()
    }

    pub fn verifier(&self) -> Arc<ServiceVerifier<T>> {
        self.verifier.clone()
    }

    pub fn prover(&self) -> Arc<ServiceProver<T>> {
        self.prover.clone()
    }

    pub fn public_did(&self) -> &str {
        self.did_exchange.public_did()
    }
}
