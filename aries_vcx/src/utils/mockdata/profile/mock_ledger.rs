use async_trait::async_trait;
use messages::{connection::did::Did, did_doc::service_aries::AriesService};

use crate::{
    error::{VcxResult, VcxErrorKind, VcxError}, plugins::ledger::base_ledger::BaseLedger,
    xyz::primitives::revocation_registry::RevocationRegistryDefinition, utils::{self, constants::{SCHEMA_JSON, rev_def_json, REV_REG_DELTA_JSON, REV_REG_ID, REV_REG_JSON, SCHEMA_TXN, CRED_DEF_JSON}}, indy::utils::LibindyMock,
};

#[derive(Debug)]
pub(super) struct MockLedger;

#[allow(unused)]
#[async_trait]
impl BaseLedger for MockLedger {
    async fn sign_and_submit_request(&self, submitter_did: &str, request_json: &str) -> VcxResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }

    async fn submit_request(&self, request_json: &str) -> VcxResult<String> {
        // not needed yet
        todo!()
    }

    async fn endorse_transaction(&self, endorser_did: &str, request_json: &str) -> VcxResult<()> {
        Ok(())
    }

    async fn set_endorser(&self, submitter_did: &str, request: &str, endorser: &str) -> VcxResult<String> {
        Ok(utils::constants::REQUEST_WITH_ENDORSER.to_string())
    }

    async fn get_txn_author_agreement(&self) -> VcxResult<String> {
        Ok(utils::constants::DEFAULT_AUTHOR_AGREEMENT.to_string())
    }

    async fn get_nym(&self, did: &str) -> VcxResult<String> {
        // not needed yet
        todo!()
    }

    async fn publish_nym(
        &self,
        submitter_did: &str,
        target_did: &str,
        verkey: Option<&str>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }

    async fn get_schema(&self, schema_id: &str, submitter_did: Option<&str>) -> VcxResult<String> {
        Ok(SCHEMA_JSON.to_string())
    }

    async fn get_cred_def(&self, cred_def_id: &str, submitter_did: Option<&str>) -> VcxResult<String> {
        // TODO FUTURE - below error is required for tests to pass which require a cred def to not exist (libvcx)
        // ideally we can migrate away from it
        let rc = LibindyMock::get_result();
        if rc == 309 {
            return Err(VcxError::from(VcxErrorKind::LibndyError(309)))
        };
        Ok(CRED_DEF_JSON.to_string())
    }

    async fn get_service(&self, did: &Did) -> VcxResult<AriesService> {
        Ok(AriesService::default())
    }

    async fn add_service(&self, did: &str, service: &AriesService) -> VcxResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }

    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxResult<String> {
        Ok(rev_def_json())
    }

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxResult<(String, String, u64)> {
        Ok((REV_REG_ID.to_string(), REV_REG_DELTA_JSON.to_string(), 1))
    }

    async fn get_rev_reg(&self, rev_reg_id: &str, timestamp: u64) -> VcxResult<(String, String, u64)> {
        Ok((REV_REG_ID.to_string(), REV_REG_JSON.to_string(), 1))
    }

    async fn get_ledger_txn(&self, seq_no: i32, submitter_did: Option<&str>) -> VcxResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }

    async fn build_schema_request(&self, submitter_did: &str, schema_json: &str) -> VcxResult<String> {
        Ok(SCHEMA_TXN.to_string())
    }

    async fn publish_schema(
        &self,
        schema_json: &str,
        submitter_did: &str,
        endorser_did: Option<String>,
    ) -> VcxResult<()> {
        Ok(())
    }

    async fn publish_cred_def(&self, cred_def_json: &str, submitter_did: &str) -> VcxResult<()> {
        Ok(())
    }

    async fn publish_rev_reg_def(
        &self,
        rev_reg_def: &RevocationRegistryDefinition,
        submitter_did: &str,
    ) -> VcxResult<()> {
        Ok(())
    }

    async fn publish_rev_reg_delta(
        &self,
        rev_reg_id: &str,
        rev_reg_entry_json: &str,
        submitter_did: &str,
    ) -> VcxResult<()> {
        Ok(())
    }
}
