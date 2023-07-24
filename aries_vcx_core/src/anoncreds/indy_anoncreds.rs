use async_trait::async_trait;

use crate::errors::error::VcxCoreResult;
use crate::{indy, WalletHandle};
use crate::wallet::indy::wallet_non_secrets::{clear_rev_reg_delta, get_rev_reg_delta};

use super::base_anoncreds::BaseAnonCreds;

#[derive(Debug)]
pub struct IndySdkAnonCreds {
    indy_wallet_handle: WalletHandle,
}

impl IndySdkAnonCreds {
    pub fn new(indy_wallet_handle: WalletHandle) -> Self {
        IndySdkAnonCreds { indy_wallet_handle }
    }
}

#[async_trait]
impl BaseAnonCreds for IndySdkAnonCreds {
    async fn verifier_verify_proof(
        &self,
        proof_req_json: &str,
        proof_json: &str,
        schemas_json: &str,
        credential_defs_json: &str,
        rev_reg_defs_json: &str,
        rev_regs_json: &str,
    ) -> VcxCoreResult<bool> {
        indy::proofs::verifier::verifier::libindy_verifier_verify_proof(
            proof_req_json,
            proof_json,
            schemas_json,
            credential_defs_json,
            rev_reg_defs_json,
            rev_regs_json,
        )
        .await
    }

    async fn issuer_create_and_store_revoc_reg(
        &self,
        issuer_did: &str,
        cred_def_id: &str,
        tails_dir: &str,
        max_creds: u32,
        tag: &str,
    ) -> VcxCoreResult<(String, String, String)> {
        indy::primitives::revocation_registry::libindy_create_and_store_revoc_reg(
            self.indy_wallet_handle,
            issuer_did,
            cred_def_id,
            tails_dir,
            max_creds,
            tag,
        )
        .await
    }

    async fn issuer_create_and_store_credential_def(
        &self,
        issuer_did: &str,
        schema_json: &str,
        tag: &str,
        sig_type: Option<&str>,
        config_json: &str,
    ) -> VcxCoreResult<(String, String)> {
        indy::primitives::credential_definition::libindy_create_and_store_credential_def(
            self.indy_wallet_handle,
            issuer_did,
            schema_json,
            tag,
            sig_type,
            config_json,
        )
        .await
    }

    async fn issuer_create_credential_offer(&self, cred_def_id: &str) -> VcxCoreResult<String> {
        indy::credentials::issuer::libindy_issuer_create_credential_offer(self.indy_wallet_handle, cred_def_id).await
    }

    async fn issuer_create_credential(
        &self,
        cred_offer_json: &str,
        cred_req_json: &str,
        cred_values_json: &str,
        rev_reg_id: Option<String>,
        tails_dir: Option<String>,
    ) -> VcxCoreResult<(String, Option<String>, Option<String>)> {
        indy::credentials::issuer::libindy_issuer_create_credential(
            self.indy_wallet_handle,
            cred_offer_json,
            cred_req_json,
            cred_values_json,
            rev_reg_id,
            tails_dir,
        )
        .await
    }

    async fn prover_create_proof(
        &self,
        proof_req_json: &str,
        requested_credentials_json: &str,
        master_secret_id: &str,
        schemas_json: &str,
        credential_defs_json: &str,
        revoc_states_json: Option<&str>,
    ) -> VcxCoreResult<String> {
        indy::proofs::prover::prover::libindy_prover_create_proof(
            self.indy_wallet_handle,
            proof_req_json,
            requested_credentials_json,
            master_secret_id,
            schemas_json,
            credential_defs_json,
            revoc_states_json,
        )
        .await
    }

    async fn prover_get_credential(&self, cred_id: &str) -> VcxCoreResult<String> {
        indy::credentials::holder::libindy_prover_get_credential(self.indy_wallet_handle, cred_id).await
    }

    async fn prover_get_credentials(&self, filter_json: Option<&str>) -> VcxCoreResult<String> {
        indy::proofs::prover::prover::libindy_prover_get_credentials(self.indy_wallet_handle, filter_json).await
    }

    async fn prover_get_credentials_for_proof_req(&self, proof_req: &str) -> VcxCoreResult<String> {
        indy::proofs::prover::prover::libindy_prover_get_credentials_for_proof_req(self.indy_wallet_handle, proof_req)
            .await
    }

    async fn prover_create_credential_req(
        &self,
        prover_did: &str,
        credential_offer_json: &str,
        credential_def_json: &str,
        master_secret_id: &str,
    ) -> VcxCoreResult<(String, String)> {
        indy::credentials::holder::libindy_prover_create_credential_req(
            self.indy_wallet_handle,
            prover_did,
            credential_offer_json,
            credential_def_json,
            master_secret_id,
        )
        .await
    }

    async fn create_revocation_state(
        &self,
        tails_dir: &str,
        rev_reg_def_json: &str,
        rev_reg_delta_json: &str,
        timestamp: u64,
        cred_rev_id: &str,
    ) -> VcxCoreResult<String> {
        indy::proofs::prover::libindy_prover_create_revocation_state(
            tails_dir,
            rev_reg_def_json,
            rev_reg_delta_json,
            timestamp,
            cred_rev_id,
        )
        .await
    }

    async fn prover_store_credential(
        &self,
        cred_id: Option<&str>,
        cred_req_meta: &str,
        cred_json: &str,
        cred_def_json: &str,
        rev_reg_def_json: Option<&str>,
    ) -> VcxCoreResult<String> {
        indy::credentials::holder::libindy_prover_store_credential(
            self.indy_wallet_handle,
            cred_id,
            cred_req_meta,
            cred_json,
            cred_def_json,
            rev_reg_def_json,
        )
        .await
    }

    async fn prover_delete_credential(&self, cred_id: &str) -> VcxCoreResult<()> {
        indy::credentials::holder::libindy_prover_delete_credential(self.indy_wallet_handle, cred_id).await
    }

    async fn prover_create_link_secret(&self, master_secret_id: &str) -> VcxCoreResult<String> {
        indy::credentials::holder::libindy_prover_create_master_secret(self.indy_wallet_handle, master_secret_id).await
    }

    async fn issuer_create_schema(
        &self,
        issuer_did: &str,
        name: &str,
        version: &str,
        attrs: &str,
    ) -> VcxCoreResult<(String, String)> {
        indy::primitives::credential_schema::libindy_issuer_create_schema(issuer_did, name, version, attrs).await
    }

    async fn revoke_credential_local(&self, tails_dir: &str, rev_reg_id: &str, cred_rev_id: &str) -> VcxCoreResult<()> {
        indy::primitives::revocation_registry::revoke_credential_local(
            self.indy_wallet_handle,
            tails_dir,
            rev_reg_id,
            cred_rev_id,
        )
        .await
    }

    async fn get_rev_reg_delta(&self, rev_reg_id: &str) -> VcxCoreResult<Option<String>> {
        Ok(get_rev_reg_delta(self.indy_wallet_handle, rev_reg_id).await)
    }

    async fn clear_rev_reg_delta(&self, rev_reg_id: &str) -> VcxCoreResult<()> {
        clear_rev_reg_delta(self.indy_wallet_handle, rev_reg_id).await?;
        Ok(())
    }

    async fn generate_nonce(&self) -> VcxCoreResult<String> {
        indy::anoncreds::generate_nonce().await
    }
}
