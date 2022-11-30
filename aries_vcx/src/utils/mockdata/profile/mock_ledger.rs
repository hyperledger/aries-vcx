use async_trait::async_trait;
use messages::{connection::did::Did, did_doc::service_aries::AriesService};

use crate::{
    error::{VcxError, VcxErrorKind, VcxResult},
    indy::utils::LibindyMock,
    plugins::ledger::base_ledger::BaseLedger,
    utils::{
        self,
        constants::{
            rev_def_json, CRED_DEF_JSON, REV_REG_DELTA_JSON, REV_REG_ID, REV_REG_JSON, SCHEMA_JSON, SCHEMA_TXN,
        },
    },
    xyz::primitives::revocation_registry::RevocationRegistryDefinition,
};

#[derive(Debug)]
pub(crate) struct MockLedger;

#[allow(unused)]
#[async_trait]
impl BaseLedger for MockLedger {
    async fn sign_and_submit_request(&self, submitter_did: &str, request_json: &str) -> VcxResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }

    async fn submit_request(&self, request_json: &str) -> VcxResult<String> {
        // not needed yet
        Err(VcxError::from_msg(
            VcxErrorKind::UnimplementedFeature,
            "unimplemented mock method",
        ))
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
        Err(VcxError::from_msg(
            VcxErrorKind::UnimplementedFeature,
            "unimplemented mock method",
        ))
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
        // TODO - FUTURE - below error is required for tests to pass which require a cred def to not exist (libvcx)
        // ideally we can migrate away from it
        let rc = LibindyMock::get_result();
        if rc == 309 {
            return Err(VcxError::from(VcxErrorKind::LibndyError(309)));
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

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {

    use crate::{
        error::{VcxErrorKind, VcxResult},
        plugins::ledger::base_ledger::BaseLedger,
    };

    use super::MockLedger;

    #[tokio::test]
    async fn test_unimplemented_methods() {
        // test used to assert which methods are unimplemented currently, can be removed after all methods implemented

        fn assert_unimplemented<T: std::fmt::Debug>(result: VcxResult<T>) {
            assert_eq!(result.unwrap_err().kind(), VcxErrorKind::UnimplementedFeature)
        }

        let ledger: Box<dyn BaseLedger> = Box::new(MockLedger);

        assert_unimplemented(ledger.submit_request("").await);
        assert_unimplemented(ledger.get_nym("").await);
    }
}
