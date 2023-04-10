use async_trait::async_trait;

use crate::errors::error::VcxCoreResult;

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
        issuer_did: &str,
        cred_def_id: &str,
        tails_dir: &str,
        max_creds: u32,
        tag: &str,
    ) -> VcxCoreResult<(String, String, String)>;

    async fn issuer_create_and_store_credential_def(
        &self,
        issuer_did: &str,
        schema_json: &str,
        tag: &str,
        signature_type: Option<&str>,
        config_json: &str,
    ) -> VcxCoreResult<(String, String)>;

    async fn issuer_create_credential_offer(&self, cred_def_id: &str) -> VcxCoreResult<String>;

    async fn issuer_create_credential(
        &self,
        cred_offer_json: &str,
        cred_req_json: &str,
        cred_values_json: &str,
        rev_reg_id: Option<String>,
        tails_dir: Option<String>,
    ) -> VcxCoreResult<(String, Option<String>, Option<String>)>;

    async fn prover_create_proof(
        &self,
        proof_req_json: &str,
        requested_credentials_json: &str,
        master_secret_id: &str,
        schemas_json: &str,
        credential_defs_json: &str,
        revoc_states_json: Option<&str>,
    ) -> VcxCoreResult<String>;

    async fn prover_get_credential(&self, cred_id: &str) -> VcxCoreResult<String>;

    async fn prover_get_credentials(&self, filter_json: Option<&str>) -> VcxCoreResult<String>;

    async fn prover_get_credentials_for_proof_req(&self, proof_request_json: &str) -> VcxCoreResult<String>;

    async fn prover_create_credential_req(
        &self,
        prover_did: &str,
        cred_offer_json: &str,
        cred_def_json: &str,
        master_secret_id: &str,
    ) -> VcxCoreResult<(String, String)>;

    async fn create_revocation_state(
        &self,
        tails_dir: &str,
        rev_reg_def_json: &str,
        rev_reg_delta_json: &str,
        timestamp: u64,
        cred_rev_id: &str,
    ) -> VcxCoreResult<String>;

    async fn prover_store_credential(
        &self,
        cred_id: Option<&str>,
        cred_req_metadata_json: &str,
        cred_json: &str,
        cred_def_json: &str,
        rev_reg_def_json: Option<&str>,
    ) -> VcxCoreResult<String>;

    async fn prover_delete_credential(&self, cred_id: &str) -> VcxCoreResult<()>;

    async fn prover_create_link_secret(&self, link_secret_id: &str) -> VcxCoreResult<String>;

    async fn issuer_create_schema(
        &self,
        issuer_did: &str,
        name: &str,
        version: &str,
        attrs: &str,
    ) -> VcxCoreResult<(String, String)>;

    // TODO - FUTURE - think about moving this to somewhere else, as it aggregates other calls (not PURE Anoncreds)
    async fn revoke_credential_local(&self, tails_dir: &str, rev_reg_id: &str, cred_rev_id: &str) -> VcxCoreResult<()>;

    // TODO - FUTURE - think about moving this to somewhere else, as it aggregates other calls (not PURE Anoncreds)
    async fn publish_local_revocations(&self, submitter_did: &str, rev_reg_id: &str) -> VcxCoreResult<()>;

    async fn generate_nonce(&self) -> VcxCoreResult<String>;
}
