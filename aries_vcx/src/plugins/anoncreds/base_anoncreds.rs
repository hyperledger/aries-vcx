use async_trait::async_trait;

use crate::error::VcxResult;

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
    ) -> VcxResult<bool>;

    async fn issuer_create_and_store_revoc_reg(
        &self,
        issuer_did: &str,
        cred_def_id: &str,
        tails_dir: &str,
        max_creds: u32,
        tag: &str
    ) -> VcxResult<(String, String, String)>;

    async fn issuer_create_and_store_credential_def(
        &self,
        issuer_did: &str,
        schema_json: &str,
        tag: &str,
        signature_type: Option<&str>,
        config_json: &str,
    ) -> VcxResult<(String, String)>;

    
    async fn issuer_create_credential_offer(
        &self, 
        cred_def_id: &str,
    ) -> VcxResult<String>;
    
    
    async fn issuer_create_credential(
        &self,
        cred_offer_json: &str,
        cred_req_json: &str,
        cred_values_json: &str,
        rev_reg_id: Option<String>,
        tails_dir: Option<String>,
    ) -> VcxResult<(String, Option<String>, Option<String>)>;

    async fn prover_create_proof(
        &self,
        proof_req_json: &str,
        requested_credentials_json: &str,
        master_secret_id: &str,
        schemas_json: &str,
        credential_defs_json: &str,
        revoc_states_json: Option<&str>,
    ) -> VcxResult<String>;

    // * `filter_json`: filter for credentials {
    //    "schema_id": string, (Optional)
    //    "schema_issuer_did": string, (Optional)
    //    "schema_name": string, (Optional)
    //    "schema_version": string, (Optional)
    //    "issuer_did": string, (Optional)
    //    "cred_def_id": string, (Optional)
    //  }

    async fn prover_get_credential(&self, cred_id: &str) -> VcxResult<String>;

    async fn prover_get_credentials(&self, filter_json: Option<&str>) -> VcxResult<String>;

    async fn prover_get_credentials_for_proof_req(&self, proof_request_json: &str) -> VcxResult<String>;

    async fn prover_create_credential_req(
        &self,
        prover_did: &str,
        cred_offer_json: &str,
        cred_def_json: &str,
        master_secret_id: &str,
    ) -> VcxResult<(String, String)>;

    async fn create_revocation_state(
        &self,
        tails_dir: &str,
        rev_reg_def_json: &str,
        rev_reg_delta_json: &str,
        timestamp: u64,
        cred_rev_id: &str,
    ) -> VcxResult<String>;

    // SKIP (unused): libindy_prover_update_revocation_state

    async fn prover_store_credential(
        &self,
        cred_id: Option<&str>,
        cred_req_metadata_json: &str,
        cred_json: &str,
        cred_def_json: &str,
        rev_reg_def_json: Option<&str>,
    ) -> VcxResult<String>;

    async fn prover_delete_credential(&self, cred_id: &str) -> VcxResult<()>;

    async fn prover_create_link_secret(&self, link_secret_id: &str) -> VcxResult<String>;

    // SKIP (internal): libindy_issuer_revoke_credential
    // SKIP (internal): libindy_issuer_merge_revocation_registry_deltas
    // SKIPO (internal): libindy_build_revoc_reg_def_request
    // SIKIP (internal): libindy_build_revoc_reg_entry_request
    // SKIP (internal): libindy_build_get_revoc_reg_def_request
    // SKIP (internal): libindy_parse_get_revoc_reg_def_response
    // SKIP (internal): libindy_build_get_revoc_reg_delta_request
    // SKLIP (internal): libindy_build_get_revoc_reg_request
    // SKIP (internal;): libindy_parse_get_revoc_reg_response
    // SKIP (internal): libindy_parse_get_cred_def_response
    // SKIP (internla ): libindy_parse_get_revoc_reg_delta_response

    async fn issuer_create_schema(
        &self,
        issuer_did: &str,
        name: &str,
        version: &str,
        attrs: &str,
    ) -> VcxResult<(String, String)>;

    // SKIP (scope): generate_cred_def // uses libindy_create_and_store_credential_def
    // SKIP (internal): build_cred_def_request

    // SKIP (scope): generate_rev_reg // uses libindy_create_and_store_revoc_reg
    // SKIP (internal): build_rev_reg_request
    // SKIP (scope): publish_rev_reg_def

    // SKIP (internal): is_cred_def_on_ledger

    // calculates new rev reg entry json and publishes to ledger (different to issuer_revoke_credential)
    // async fn revoke_credential_and_publish(
    //     &self,
    //     tails_file: &str,
    //     rev_reg_id: &str,
    //     cred_rev_id: &str,
    // ) -> VcxResult<String>;
    
    // todo - move?
    async fn revoke_credential_local(
        &self,
        tails_dir: &str,
        rev_reg_id: &str,
        cred_rev_id: &str,
    ) -> VcxResult<()>;
    
    // todo - move?
    async fn publish_local_revocations(&self, submitter_did: &str, rev_reg_id: &str) -> VcxResult<()>;

    // SKIP (internal): libindy_to_unqualified
    // SKIP{ (internla)}: libindy_build_get_txn_request
    // SKIP Internal: build_get_txn_request

    
    // SKIP (tineral): _check_schema_response
    // SKIP (Internal): _check_response

    async fn generate_nonce(&self) -> VcxResult<String>;
}
