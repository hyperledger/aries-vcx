use async_trait::async_trait;

use crate::{
    error::{VcxError, VcxErrorKind, VcxResult},
    indy::utils::LibindyMock,
    plugins::anoncreds::base_anoncreds::BaseAnonCreds,
    utils::{self, constants::{LIBINDY_CRED_OFFER, REV_STATE_JSON, LARGE_NONCE}, mockdata::mock_settings::get_mock_creds_retrieved_for_proof_request}, global::settings,
};

#[derive(Debug)]
pub(super) struct MockAnoncreds;

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
    ) -> VcxResult<bool> {
        Err(VcxError::from_msg(VcxErrorKind::UnimplementedFeature, "unimplemented mock method"))
    }

    async fn issuer_create_and_store_revoc_reg(
        &self,
        _issuer_did: &str,
        _cred_def_id: &str,
        _tails_dir: &str,
        _max_creds: u32,
        _tag: &str,
    ) -> VcxResult<(String, String, String)> {
        // not needed yet
        todo!()
    }

    async fn issuer_create_and_store_credential_def(
        &self,
        _issuer_did: &str,
        _schema_json: &str,
        _tag: &str,
        _signature_type: Option<&str>,
        _config_json: &str,
    ) -> VcxResult<(String, String)> {
        // not needed yet
        todo!()
    }

    async fn issuer_create_credential_offer(&self, _cred_def_id: &str) -> VcxResult<String> {
        let rc = LibindyMock::get_result();
        if rc != 0 {
            return Err(VcxError::from(VcxErrorKind::InvalidState));
        };
        Ok(LIBINDY_CRED_OFFER.to_string())
    }

    async fn issuer_create_credential(
        &self,
        _cred_offer_json: &str,
        _cred_req_json: &str,
        _cred_values_json: &str,
        _rev_reg_id: Option<String>,
        _tails_dir: Option<String>,
    ) -> VcxResult<(String, Option<String>, Option<String>)> {
        Ok((utils::constants::CREDENTIAL_JSON.to_owned(), None, None))
    }

    async fn prover_create_proof(
        &self,
        _proof_req_json: &str,
        _requested_credentials_json: &str,
        _master_secret_id: &str,
        _schemas_json: &str,
        _credential_defs_json: &str,
        _revoc_states_json: Option<&str>,
    ) -> VcxResult<String> {
        Ok(utils::constants::PROOF_JSON.to_owned())
    }

    async fn prover_get_credential(&self, _cred_id: &str) -> VcxResult<String> {
        // not needed yet
        todo!()
    }

    async fn prover_get_credentials(&self, _filter_json: Option<&str>) -> VcxResult<String> {
        // not needed yet
        todo!()
    }

    async fn prover_get_credentials_for_proof_req(&self, _proof_request_json: &str) -> VcxResult<String> {
        match get_mock_creds_retrieved_for_proof_request() {
            None => Err(VcxError::from_msg(VcxErrorKind::UnimplementedFeature, "mock data for `prover_get_credentials_for_proof_req` must be set")),
            Some(mocked_creds) => {
                warn!("get_mock_creds_retrieved_for_proof_request  returning mocked response");
                Ok(mocked_creds)
            }
        }
    }

    async fn prover_create_credential_req(
        &self,
        _prover_did: &str,
        _cred_offer_json: &str,
        _cred_def_json: &str,
        _master_secret_id: &str,
    ) -> VcxResult<(String, String)> {
        Ok((utils::constants::CREDENTIAL_REQ_STRING.to_owned(), String::new()))
    }

    async fn create_revocation_state(
        &self,
        _tails_dir: &str,
        _rev_reg_def_json: &str,
        _rev_reg_delta_json: &str,
        _timestamp: u64,
        _cred_rev_id: &str,
    ) -> VcxResult<String> {
        Ok(REV_STATE_JSON.to_string())
    }

    async fn prover_store_credential(
        &self,
        _cred_id: Option<&str>,
        _cred_req_metadata_json: &str,
        _cred_json: &str,
        _cred_def_json: &str,
        _rev_reg_def_json: Option<&str>,
    ) -> VcxResult<String> {
        Ok("cred_id".to_string())
    }

    async fn prover_delete_credential(&self, _cred_id: &str) -> VcxResult<()> {
        // not needed yet
        todo!()
    }

    async fn prover_create_link_secret(&self, _link_secret_id: &str) -> VcxResult<String> {
        Ok(settings::DEFAULT_LINK_SECRET_ALIAS.to_string())
    }

    async fn issuer_create_schema(
        &self,
        _issuer_did: &str,
        _name: &str,
        _version: &str,
        _attrs: &str,
    ) -> VcxResult<(String, String)> {
        // not needed yet
        todo!()
    }

    async fn revoke_credential_local(&self, _tails_dir: &str, _rev_reg_id: &str, _cred_rev_id: &str) -> VcxResult<()> {
        Ok(())
    }

    async fn publish_local_revocations(&self, _submitter_did: &str, _rev_reg_id: &str) -> VcxResult<()> {
        Ok(())
    }

    async fn generate_nonce(&self) -> VcxResult<String> {
        Ok(LARGE_NONCE.to_string())
    }
}
