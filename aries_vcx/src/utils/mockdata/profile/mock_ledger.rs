use crate::utils::mockdata::mock_settings::StatusCodeMock;
use aries_vcx_core::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};
use aries_vcx_core::ledger::base_ledger::BaseLedger;
use async_trait::async_trait;

use crate::utils::{
    self,
    constants::{rev_def_json, CRED_DEF_JSON, REV_REG_DELTA_JSON, REV_REG_ID, REV_REG_JSON, SCHEMA_JSON, SCHEMA_TXN},
};

#[derive(Debug)]
pub(crate) struct MockLedger;

// NOTE : currently matches the expected results if indy_mocks are enabled
/// Implementation of [BaseLedger] which responds with mock data
#[allow(unused)]
#[async_trait]
impl BaseLedger for MockLedger {
    async fn sign_and_submit_request(&self, submitter_did: &str, request_json: &str) -> VcxCoreResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }

    async fn submit_request(&self, request_json: &str) -> VcxCoreResult<String> {
        // not needed yet
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "unimplemented mock method: submit_request",
        ))
    }

    async fn endorse_transaction(&self, endorser_did: &str, request_json: &str) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn set_endorser(&self, submitter_did: &str, request: &str, endorser: &str) -> VcxCoreResult<String> {
        Ok(utils::constants::REQUEST_WITH_ENDORSER.to_string())
    }

    async fn get_txn_author_agreement(&self) -> VcxCoreResult<String> {
        Ok(utils::constants::DEFAULT_AUTHOR_AGREEMENT.to_string())
    }

    async fn get_nym(&self, did: &str) -> VcxCoreResult<String> {
        // not needed yet
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "unimplemented mock method: get_nym",
        ))
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

    async fn get_schema(&self, schema_id: &str, submitter_did: Option<&str>) -> VcxCoreResult<String> {
        Ok(SCHEMA_JSON.to_string())
    }

    async fn get_cred_def(&self, cred_def_id: &str, submitter_did: Option<&str>) -> VcxCoreResult<String> {
        // TODO - FUTURE - below error is required for tests to pass which require a cred def to not exist (libvcx)
        // ideally we can migrate away from it
        if StatusCodeMock::get_result() == 309 {
            return Err(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::LedgerItemNotFound,
                "Mocked error".to_string(),
            ));
        };
        Ok(CRED_DEF_JSON.to_string())
    }

    async fn get_attr(&self, target_did: &str, attr_name: &str) -> VcxCoreResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }

    async fn add_attr(&self, target_did: &str, attrib_json: &str) -> VcxCoreResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
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

    async fn get_rev_reg(&self, rev_reg_id: &str, timestamp: u64) -> VcxCoreResult<(String, String, u64)> {
        Ok((REV_REG_ID.to_string(), REV_REG_JSON.to_string(), 1))
    }

    async fn get_ledger_txn(&self, seq_no: i32, submitter_did: Option<&str>) -> VcxCoreResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }

    async fn build_schema_request(&self, submitter_did: &str, schema_json: &str) -> VcxCoreResult<String> {
        Ok(SCHEMA_TXN.to_string())
    }

    async fn publish_schema(
        &self,
        schema_json: &str,
        submitter_did: &str,
        endorser_did: Option<String>,
    ) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn publish_cred_def(&self, cred_def_json: &str, submitter_did: &str) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn publish_rev_reg_def(&self, rev_reg_def: &str, submitter_did: &str) -> VcxCoreResult<()> {
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
pub mod mocks {
    use aries_vcx_core::ledger::base_ledger::BaseLedger;
    use mockall::mock;

    mock! {
        #[derive(Debug)]
        pub MockAllLedger {}
        impl BaseLedger for MockAllLedger {
            fn sign_and_submit_request<'life0,'life1,'life2,'async_trait>(&'life0 self,submitter_did: &'life1 str,request_json: &'life2 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,Self:'async_trait;
            fn submit_request<'life0,'life1,'async_trait>(&'life0 self,request_json: &'life1 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait;
            fn endorse_transaction<'life0,'life1,'life2,'async_trait>(&'life0 self,endorser_did: &'life1 str,request_json: &'life2 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<()> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,Self:'async_trait;
            fn set_endorser<'life0,'life1,'life2,'life3,'async_trait>(&'life0 self,submitter_did: &'life1 str,request: &'life2 str,endorser: &'life3 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,Self:'async_trait;
            fn get_txn_author_agreement<'life0,'async_trait>(&'life0 self) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait;
            fn get_nym<'life0,'life1,'async_trait>(&'life0 self,did: &'life1 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait;
            fn publish_nym<'life0,'life1,'life2,'life3,'life4,'life5,'async_trait>(&'life0 self,submitter_did: &'life1 str,target_did: &'life2 str,verkey:Option< &'life3 str> ,data:Option< &'life4 str> ,role:Option< &'life5 str> ,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,'life4:'async_trait,'life5:'async_trait,Self:'async_trait;
            fn get_schema<'life0,'life1,'life2,'async_trait>(&'life0 self,schema_id: &'life1 str,submitter_did:Option< &'life2 str>) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,Self:'async_trait;
            fn get_cred_def<'life0,'life1,'life2,'async_trait>(&'life0 self,cred_def_id: &'life1 str,submitter_did:Option< &'life2 str>) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,Self:'async_trait;
            fn get_attr<'life0,'life1,'life2,'async_trait>(&'life0 self,target_did: &'life1 str,attr_name: &'life2 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,Self:'async_trait;
            fn add_attr<'life0,'life1,'life2,'async_trait>(&'life0 self,target_did: &'life1 str,attrib_json: &'life2 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,Self:'async_trait;
            fn get_rev_reg_def_json<'life0,'life1,'async_trait>(&'life0 self,rev_reg_id: &'life1 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait;
            fn get_rev_reg_delta_json<'life0,'life1,'async_trait>(&'life0 self,rev_reg_id: &'life1 str,from:Option<u64> ,to:Option<u64> ,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<(String,String,u64)> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait;
            fn get_rev_reg<'life0,'life1,'async_trait>(&'life0 self,rev_reg_id: &'life1 str,timestamp:u64) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<(String,String,u64)> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait;
            fn get_ledger_txn<'life0,'life1,'async_trait>(&'life0 self,seq_no:i32,submitter_did:Option< &'life1 str>) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait;
            fn build_schema_request<'life0,'life1,'life2,'async_trait>(&'life0 self,submitter_did: &'life1 str,schema_json: &'life2 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,Self:'async_trait;
            fn publish_schema<'life0,'life1,'life2,'async_trait>(&'life0 self,schema_json: &'life1 str,submitter_did: &'life2 str,endorser_did:Option<String> ,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<()> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,Self:'async_trait;
            fn publish_cred_def<'life0,'life1,'life2,'async_trait>(&'life0 self,cred_def_json: &'life1 str,submitter_did: &'life2 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<()> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,Self:'async_trait;
            fn publish_rev_reg_def<'life0,'life1,'life2,'async_trait>(&'life0 self,rev_reg_def: &'life1 str,submitter_did: &'life2 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<()> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,Self:'async_trait;
            fn publish_rev_reg_delta<'life0,'life1,'life2,'life3,'async_trait>(&'life0 self,rev_reg_id: &'life1 str,rev_reg_entry_json: &'life2 str,submitter_did: &'life3 str,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<()> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,Self:'async_trait;
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod unit_tests {

    use aries_vcx_core::{
        errors::error::{AriesVcxCoreErrorKind, VcxCoreResult},
        ledger::base_ledger::BaseLedger,
    };

    use super::MockLedger;

    #[tokio::test]
    async fn test_unimplemented_methods() {
        // test used to assert which methods are unimplemented currently, can be removed after all methods implemented

        fn assert_unimplemented<T: std::fmt::Debug>(result: VcxCoreResult<T>) {
            assert_eq!(result.unwrap_err().kind(), AriesVcxCoreErrorKind::UnimplementedFeature)
        }

        let ledger: Box<dyn BaseLedger> = Box::new(MockLedger);

        assert_unimplemented(ledger.submit_request("").await);
        assert_unimplemented(ledger.get_nym("").await);
    }
}
