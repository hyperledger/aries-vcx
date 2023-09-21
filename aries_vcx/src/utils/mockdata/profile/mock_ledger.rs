use aries_vcx_core::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    ledger::{
        base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite},
        indy_vdr_ledger::UpdateRole,
    },
};
use async_trait::async_trait;

use crate::{
    utils,
    utils::constants::{
        rev_def_json, CRED_DEF_JSON, REV_REG_DELTA_JSON, REV_REG_ID, REV_REG_JSON, SCHEMA_JSON,
    },
};

#[derive(Debug)]
pub struct MockLedger;

#[allow(unused)]
#[async_trait]
impl IndyLedgerRead for MockLedger {
    async fn get_txn_author_agreement(&self) -> VcxCoreResult<Option<String>> {
        Ok(Some(utils::constants::DEFAULT_AUTHOR_AGREEMENT.to_string()))
    }

    async fn get_nym(&self, did: &str) -> VcxCoreResult<String> {
        // not needed yet
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "unimplemented mock method: get_nym",
        ))
    }

    async fn get_attr(&self, target_did: &str, attr_name: &str) -> VcxCoreResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }

    async fn get_ledger_txn(
        &self,
        seq_no: i32,
        submitter_did: Option<&str>,
    ) -> VcxCoreResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }
}

#[allow(unused)]
#[async_trait]
impl IndyLedgerWrite for MockLedger {
    async fn set_endorser(
        &self,
        submitter_did: &str,
        request: &str,
        endorser: &str,
    ) -> VcxCoreResult<String> {
        Ok(utils::constants::REQUEST_WITH_ENDORSER.to_string())
    }

    async fn endorse_transaction(
        &self,
        endorser_did: &str,
        request_json: &str,
    ) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn publish_nym(
        &self,
        submitter_did: &str,
        target_did: &str,
        verkey: Option<&str>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxCoreResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }

    async fn add_attr(&self, target_did: &str, attrib_json: &str) -> VcxCoreResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }

    async fn write_did(
        &self,
        submitter_did: &str,
        target_did: &str,
        target_vk: &str,
        role: Option<UpdateRole>,
        alias: Option<String>,
    ) -> VcxCoreResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }
}

#[allow(unused)]
#[async_trait]
impl AnoncredsLedgerRead for MockLedger {
    async fn get_schema(
        &self,
        schema_id: &str,
        submitter_did: Option<&str>,
    ) -> VcxCoreResult<String> {
        Ok(SCHEMA_JSON.to_string())
    }

    async fn get_cred_def(
        &self,
        cred_def_id: &str,
        submitter_did: Option<&str>,
    ) -> VcxCoreResult<String> {
        Ok(CRED_DEF_JSON.to_string())
    }

    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxCoreResult<String> {
        Ok(rev_def_json())
    }

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxCoreResult<(String, String, u64)> {
        Ok((REV_REG_ID.to_string(), REV_REG_DELTA_JSON.to_string(), 1))
    }

    async fn get_rev_reg(
        &self,
        rev_reg_id: &str,
        timestamp: u64,
    ) -> VcxCoreResult<(String, String, u64)> {
        Ok((REV_REG_ID.to_string(), REV_REG_JSON.to_string(), 1))
    }
}

#[allow(unused)]
#[async_trait]
impl AnoncredsLedgerWrite for MockLedger {
    async fn publish_schema(
        &self,
        schema_json: &str,
        submitter_did: &str,
        endorser_did: Option<String>,
    ) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn publish_cred_def(
        &self,
        cred_def_json: &str,
        submitter_did: &str,
    ) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn publish_rev_reg_def(
        &self,
        rev_reg_def: &str,
        submitter_did: &str,
    ) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn publish_rev_reg_delta(
        &self,
        rev_reg_id: &str,
        rev_reg_entry_json: &str,
        submitter_did: &str,
    ) -> VcxCoreResult<()> {
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod unit_tests {

    use aries_vcx_core::{
        errors::error::{AriesVcxCoreErrorKind, VcxCoreResult},
        ledger::base_ledger::IndyLedgerRead,
    };

    use super::MockLedger;

    #[tokio::test]
    async fn test_unimplemented_methods() {
        // test used to assert which methods are unimplemented currently, can be removed after all
        // methods implemented

        fn assert_unimplemented<T: std::fmt::Debug>(result: VcxCoreResult<T>) {
            assert_eq!(
                result.unwrap_err().kind(),
                AriesVcxCoreErrorKind::UnimplementedFeature
            )
        }

        let ledger: Box<dyn IndyLedgerRead> = Box::new(MockLedger);

        assert_unimplemented(ledger.get_nym("").await);
    }
}
