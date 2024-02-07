use std::path::Path;

use anoncreds_types::data_types::{
    identifiers::{
        cred_def_id::CredentialDefinitionId, rev_reg_def_id::RevocationRegistryDefinitionId,
        schema_id::SchemaId,
    },
    ledger::{
        cred_def::CredentialDefinition, rev_reg::RevocationRegistry,
        rev_reg_def::RevocationRegistryDefinition, rev_reg_delta::RevocationRegistryDelta,
        schema::Schema,
    },
    messages::{cred_offer::CredentialOffer, cred_request::CredentialRequest, nonce::Nonce},
};
use async_trait::async_trait;
use did_parser::Did;

use crate::{errors::error::VcxCoreResult, wallet::base_wallet::BaseWallet};

/// Trait defining standard 'anoncreds' related functionality. The APIs, including
/// input and output types are based off the indy Anoncreds API:
/// see: <https://github.com/hyperledger/indy-sdk/blob/main/libindy/src/api/anoncreds.rs>
#[async_trait]
pub trait BaseAnonCreds: std::fmt::Debug + Send + Sync {
    async fn verifier_verify_proof(
        &self,
        proof_request_json: &str,
        proof_json: &str,
        schemas_json: &str,
        credential_defs_json: &str,
        rev_reg_defs_json: &str,
        rev_regs_json: &str,
    ) -> VcxCoreResult<bool>;

    async fn issuer_create_and_store_revoc_reg(
        &self,
        wallet: &impl BaseWallet,
        issuer_did: &Did,
        cred_def_id: &CredentialDefinitionId,
        tails_dir: &Path,
        max_creds: u32,
        tag: &str,
    ) -> VcxCoreResult<(
        RevocationRegistryDefinitionId,
        RevocationRegistryDefinition,
        RevocationRegistry,
    )>;

    #[allow(clippy::too_many_arguments)]
    async fn issuer_create_and_store_credential_def(
        &self,
        wallet: &impl BaseWallet,
        issuer_did: &Did,
        schema_id: &SchemaId,
        schema_json: Schema,
        tag: &str,
        signature_type: Option<&str>,
        config_json: &str,
    ) -> VcxCoreResult<CredentialDefinition>;

    async fn issuer_create_credential_offer(
        &self,
        wallet: &impl BaseWallet,
        cred_def_id: &CredentialDefinitionId,
    ) -> VcxCoreResult<CredentialOffer>;

    async fn issuer_create_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_offer_json: CredentialOffer,
        cred_req_json: CredentialRequest,
        cred_values_json: &str,
        rev_reg_id: Option<&RevocationRegistryDefinitionId>,
        tails_dir: Option<&Path>,
    ) -> VcxCoreResult<(String, Option<String>)>;

    #[allow(clippy::too_many_arguments)]
    async fn prover_create_proof(
        &self,
        wallet: &impl BaseWallet,
        proof_req_json: &str,
        requested_credentials_json: &str,
        master_secret_id: &str,
        schemas_json: &str,
        credential_defs_json: &str,
        revoc_states_json: Option<&str>,
    ) -> VcxCoreResult<String>;

    async fn prover_get_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_id: &str,
    ) -> VcxCoreResult<String>;

    async fn prover_get_credentials(
        &self,
        wallet: &impl BaseWallet,
        filter_json: Option<&str>,
    ) -> VcxCoreResult<String>;

    async fn prover_get_credentials_for_proof_req(
        &self,
        wallet: &impl BaseWallet,
        proof_request_json: &str,
    ) -> VcxCoreResult<String>;

    async fn prover_create_credential_req(
        &self,
        wallet: &impl BaseWallet,
        prover_did: &Did,
        cred_offer_json: &str,
        cred_def_json: CredentialDefinition,
        master_secret_id: &str,
    ) -> VcxCoreResult<(String, String)>;

    async fn create_revocation_state(
        &self,
        tails_dir: &Path,
        rev_reg_def_json: RevocationRegistryDefinition,
        rev_reg_delta_json: RevocationRegistryDelta,
        timestamp: u64,
        cred_rev_id: u32,
    ) -> VcxCoreResult<String>;

    async fn prover_store_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_id: Option<&str>,
        cred_req_metadata_json: &str,
        cred_json: &str,
        cred_def_json: CredentialDefinition,
        rev_reg_def_json: Option<RevocationRegistryDefinition>,
    ) -> VcxCoreResult<String>;

    async fn prover_delete_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_id: &str,
    ) -> VcxCoreResult<()>;

    async fn prover_create_link_secret(
        &self,
        wallet: &impl BaseWallet,
        link_secret_id: &str,
    ) -> VcxCoreResult<()>;

    async fn issuer_create_schema(
        &self,
        issuer_did: &Did,
        name: &str,
        version: &str,
        attrs: &str,
    ) -> VcxCoreResult<Schema>;

    // TODO - FUTURE - think about moving this to somewhere else, as it aggregates other calls (not
    // PURE Anoncreds)
    // ^ YES
    async fn revoke_credential_local(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_id: &RevocationRegistryDefinitionId,
        cred_rev_id: u32,
        rev_reg_delta_json: RevocationRegistryDelta,
    ) -> VcxCoreResult<()>;

    async fn get_rev_reg_delta(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_id: &RevocationRegistryDefinitionId,
    ) -> VcxCoreResult<Option<RevocationRegistryDelta>>;

    async fn clear_rev_reg_delta(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_id: &RevocationRegistryDefinitionId,
    ) -> VcxCoreResult<()>;

    async fn generate_nonce(&self) -> VcxCoreResult<Nonce>;
}
