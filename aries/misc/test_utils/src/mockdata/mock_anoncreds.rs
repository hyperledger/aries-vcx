use anoncreds_types::data_types::ledger::schema::Schema;
use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    global::settings::DEFAULT_LINK_SECRET_ALIAS,
    wallet::base_wallet::BaseWallet,
};
use async_trait::async_trait;

use crate::constants::{
    CREDENTIAL_JSON, CREDENTIAL_REQ_STRING, LARGE_NONCE, LIBINDY_CRED_OFFER, PROOF_JSON,
    REV_REG_DELTA_JSON, REV_STATE_JSON,
};

#[derive(Debug)]
pub struct MockAnoncreds;

// NOTE : currently matches the expected results if indy_mocks are enabled
/// Implementation of [BaseAnoncreds] which responds with mock data
#[async_trait]
impl BaseAnonCreds for MockAnoncreds {
    async fn verifier_verify_proof(
        &self,
        _proof_request_json: &str,
        _proof_json: &str,
        _schemas_json: &str,
        _credential_defs_json: &str,
        _rev_reg_defs_json: &str,
        _rev_regs_json: &str,
    ) -> VcxCoreResult<bool> {
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "unimplemented mock method: verifier_verify_proof",
        ))
    }

    async fn issuer_create_and_store_revoc_reg(
        &self,
        __wallet: &impl BaseWallet,
        _issuer_did: &str,
        _cred_def_id: &str,
        _tails_dir: &str,
        _max_creds: u32,
        _tag: &str,
    ) -> VcxCoreResult<(String, String, String)> {
        // not needed yet
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "unimplemented mock method: issuer_create_and_store_revoc_reg",
        ))
    }

    async fn issuer_create_and_store_credential_def(
        &self,
        __wallet: &impl BaseWallet,
        _issuer_did: &str,
        _schema_id: &str,
        _schema_json: Schema,
        _tag: &str,
        _signature_type: Option<&str>,
        _config_json: &str,
    ) -> VcxCoreResult<(String, String)> {
        // not needed yet
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "unimplemented mock method: issuer_create_and_store_credential_def",
        ))
    }

    async fn issuer_create_credential_offer(
        &self,
        __wallet: &impl BaseWallet,
        _cred_def_id: &str,
    ) -> VcxCoreResult<String> {
        Ok(LIBINDY_CRED_OFFER.to_string())
    }

    async fn issuer_create_credential(
        &self,
        __wallet: &impl BaseWallet,
        _cred_offer_json: &str,
        _cred_req_json: &str,
        _cred_values_json: &str,
        _rev_reg_id: Option<String>,
        _tails_dir: Option<String>,
    ) -> VcxCoreResult<(String, Option<String>, Option<String>)> {
        Ok((CREDENTIAL_JSON.to_owned(), None, None))
    }

    async fn prover_create_proof(
        &self,
        __wallet: &impl BaseWallet,
        _proof_req_json: &str,
        _requested_credentials_json: &str,
        _master_secret_id: &str,
        _schemas_json: &str,
        _credential_defs_json: &str,
        _revoc_states_json: Option<&str>,
    ) -> VcxCoreResult<String> {
        Ok(PROOF_JSON.to_owned())
    }

    async fn prover_get_credential(
        &self,
        __wallet: &impl BaseWallet,
        _cred_id: &str,
    ) -> VcxCoreResult<String> {
        // not needed yet
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "unimplemented mock method: prover_get_credential",
        ))
    }

    async fn prover_get_credentials(
        &self,
        __wallet: &impl BaseWallet,
        _filter_json: Option<&str>,
    ) -> VcxCoreResult<String> {
        // not needed yet
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "unimplemented mock method: prover_get_credentials",
        ))
    }

    async fn prover_get_credentials_for_proof_req(
        &self,
        _wallet: &impl BaseWallet,
        _proof_request_json: &str,
    ) -> VcxCoreResult<String> {
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "mock data for `prover_get_credentials_for_proof_req` must be set",
        ))
    }

    // todo: change _prover_did argument, see: https://github.com/hyperledger/aries-vcx/issues/950
    async fn prover_create_credential_req(
        &self,
        _wallet: &impl BaseWallet,
        _prover_did: &str,
        _cred_offer_json: &str,
        _cred_def_json: &str,
        _master_secret_id: &str,
    ) -> VcxCoreResult<(String, String)> {
        Ok((CREDENTIAL_REQ_STRING.to_owned(), String::new()))
    }

    async fn create_revocation_state(
        &self,
        _tails_dir: &str,
        _rev_reg_def_json: &str,
        _rev_reg_delta_json: &str,
        _timestamp: u64,
        _cred_rev_id: &str,
    ) -> VcxCoreResult<String> {
        Ok(REV_STATE_JSON.to_string())
    }

    async fn prover_store_credential(
        &self,
        _wallet: &impl BaseWallet,
        _cred_id: Option<&str>,
        _cred_req_metadata_json: &str,
        _cred_json: &str,
        _cred_def_json: &str,
        _rev_reg_def_json: Option<&str>,
    ) -> VcxCoreResult<String> {
        Ok("cred_id".to_string())
    }

    async fn prover_delete_credential(
        &self,
        _wallet: &impl BaseWallet,
        _cred_id: &str,
    ) -> VcxCoreResult<()> {
        // not needed yet
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "unimplemented mock method: prover_delete_credential",
        ))
    }

    async fn prover_create_link_secret(
        &self,
        _wallet: &impl BaseWallet,
        _link_secret_id: &str,
    ) -> VcxCoreResult<String> {
        Ok(DEFAULT_LINK_SECRET_ALIAS.to_string())
    }

    async fn issuer_create_schema(
        &self,
        _issuer_did: &str,
        _name: &str,
        _version: &str,
        _attrs: &str,
    ) -> VcxCoreResult<Schema> {
        // not needed yet
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "unimplemented mock method: issuer_create_schema",
        ))
    }

    async fn revoke_credential_local(
        &self,
        _wallet: &impl BaseWallet,
        _tails_dir: &str,
        _rev_reg_id: &str,
        _cred_rev_id: &str,
        _rev_reg_delta_json: &str,
    ) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn get_rev_reg_delta(
        &self,
        _wallet: &impl BaseWallet,
        _rev_reg_id: &str,
    ) -> VcxCoreResult<Option<String>> {
        Ok(Some(REV_REG_DELTA_JSON.to_string()))
    }

    async fn clear_rev_reg_delta(
        &self,
        _wallet: &impl BaseWallet,
        _rev_reg_id: &str,
    ) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn generate_nonce(&self) -> VcxCoreResult<String> {
        Ok(LARGE_NONCE.to_string())
    }
}
