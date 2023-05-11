use crate::utils::mockdata::mock_settings::StatusCodeMock;
use aries_vcx_core::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};
use async_trait::async_trait;

use crate::{
    global::settings,
    utils::{
        self,
        constants::{LARGE_NONCE, LIBINDY_CRED_OFFER, REV_STATE_JSON},
        mockdata::mock_settings::get_mock_creds_retrieved_for_proof_request,
    },
};
use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;

#[derive(Debug)]
pub(crate) struct MockAnoncreds;

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
        _issuer_did: &str,
        _schema_json: &str,
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

    async fn issuer_create_credential_offer(&self, _cred_def_id: &str) -> VcxCoreResult<String> {
        if StatusCodeMock::get_result() != 0 {
            return Err(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidState,
                "Mocked error result of issuer_create_credential_offer: issuer_create_credential_offer",
            ));
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
    ) -> VcxCoreResult<(String, Option<String>, Option<String>)> {
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
    ) -> VcxCoreResult<String> {
        Ok(utils::constants::PROOF_JSON.to_owned())
    }

    async fn prover_get_credential(&self, _cred_id: &str) -> VcxCoreResult<String> {
        // not needed yet
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "unimplemented mock method: prover_get_credential",
        ))
    }

    async fn prover_get_credentials(&self, _filter_json: Option<&str>) -> VcxCoreResult<String> {
        // not needed yet
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "unimplemented mock method: prover_get_credentials",
        ))
    }

    async fn prover_get_credentials_for_proof_req(&self, _proof_request_json: &str) -> VcxCoreResult<String> {
        match get_mock_creds_retrieved_for_proof_request() {
            None => Err(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::UnimplementedFeature,
                "mock data for `prover_get_credentials_for_proof_req` must be set",
            )),
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
    ) -> VcxCoreResult<(String, String)> {
        Ok((utils::constants::CREDENTIAL_REQ_STRING.to_owned(), String::new()))
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
        _cred_id: Option<&str>,
        _cred_req_metadata_json: &str,
        _cred_json: &str,
        _cred_def_json: &str,
        _rev_reg_def_json: Option<&str>,
    ) -> VcxCoreResult<String> {
        Ok("cred_id".to_string())
    }

    async fn prover_delete_credential(&self, _cred_id: &str) -> VcxCoreResult<()> {
        // not needed yet
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "unimplemented mock method: prover_delete_credential",
        ))
    }

    async fn prover_create_link_secret(&self, _link_secret_id: &str) -> VcxCoreResult<String> {
        Ok(settings::DEFAULT_LINK_SECRET_ALIAS.to_string())
    }

    async fn issuer_create_schema(
        &self,
        _issuer_did: &str,
        _name: &str,
        _version: &str,
        _attrs: &str,
    ) -> VcxCoreResult<(String, String)> {
        // not needed yet
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "unimplemented mock method: issuer_create_schema",
        ))
    }

    async fn revoke_credential_local(
        &self,
        _tails_dir: &str,
        _rev_reg_id: &str,
        _cred_rev_id: &str,
    ) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn publish_local_revocations(&self, _submitter_did: &str, _rev_reg_id: &str) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn generate_nonce(&self) -> VcxCoreResult<String> {
        Ok(LARGE_NONCE.to_string())
    }
}

#[cfg(test)]
pub mod mocks {
    use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
    use mockall::mock;

    mock! {
        #[derive(Debug)]
        pub MockAllAnonCreds {}

        impl BaseAnonCreds for MockAllAnonCreds {
            fn verifier_verify_proof<'life0,'life1,'life2,'life3,'life4,'life5,'life6,'async_trait>(&'life0 self,proof_request_json: &'life1 str,proof_json: &'life2 str,schemas_json: &'life3 str,credential_defs_json: &'life4 str,rev_reg_defs_json: &'life5 str,rev_regs_json: &'life6 str,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<bool> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,'life4:'async_trait,'life5:'async_trait,'life6:'async_trait,Self:'async_trait;
            fn issuer_create_and_store_revoc_reg<'life0,'life1,'life2,'life3,'life4,'async_trait>(&'life0 self,issuer_did: &'life1 str,cred_def_id: &'life2 str,tails_dir: &'life3 str,max_creds:u32,tag: &'life4 str,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<(String,String,String)> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,'life4:'async_trait,Self:'async_trait;
            fn issuer_create_and_store_credential_def<'life0,'life1,'life2,'life3,'life4,'life5,'async_trait>(&'life0 self,issuer_did: &'life1 str,schema_json: &'life2 str,tag: &'life3 str,signature_type:Option< &'life4 str> ,config_json: &'life5 str,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<(String,String)> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,'life4:'async_trait,'life5:'async_trait,Self:'async_trait;
            fn issuer_create_credential_offer<'life0,'life1,'async_trait>(&'life0 self,cred_def_id: &'life1 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait;
            fn issuer_create_credential<'life0,'life1,'life2,'life3,'async_trait>(&'life0 self,cred_offer_json: &'life1 str,cred_req_json: &'life2 str,cred_values_json: &'life3 str,rev_reg_id:Option<String> ,tails_dir:Option<String> ,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<(String,Option<String> ,Option<String>)> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,Self:'async_trait;
            fn prover_create_proof<'life0,'life1,'life2,'life3,'life4,'life5,'life6,'async_trait>(&'life0 self,proof_req_json: &'life1 str,requested_credentials_json: &'life2 str,master_secret_id: &'life3 str,schemas_json: &'life4 str,credential_defs_json: &'life5 str,revoc_states_json:Option< &'life6 str> ,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,'life4:'async_trait,'life5:'async_trait,'life6:'async_trait,Self:'async_trait;
            fn prover_get_credential<'life0,'life1,'async_trait>(&'life0 self,cred_id: &'life1 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait;
            fn prover_get_credentials<'life0,'life1,'async_trait>(&'life0 self,filter_json:Option< &'life1 str>) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait;
            fn prover_get_credentials_for_proof_req<'life0,'life1,'async_trait>(&'life0 self,proof_request_json: &'life1 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait;
            fn prover_create_credential_req<'life0,'life1,'life2,'life3,'life4,'async_trait>(&'life0 self,prover_did: &'life1 str,cred_offer_json: &'life2 str,cred_def_json: &'life3 str,master_secret_id: &'life4 str,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<(String,String)> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,'life4:'async_trait,Self:'async_trait;
            fn create_revocation_state<'life0,'life1,'life2,'life3,'life4,'async_trait>(&'life0 self,tails_dir: &'life1 str,rev_reg_def_json: &'life2 str,rev_reg_delta_json: &'life3 str,timestamp:u64,cred_rev_id: &'life4 str,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,'life4:'async_trait,Self:'async_trait;
            fn prover_store_credential<'life0,'life1,'life2,'life3,'life4,'life5,'async_trait>(&'life0 self,cred_id:Option< &'life1 str> ,cred_req_metadata_json: &'life2 str,cred_json: &'life3 str,cred_def_json: &'life4 str,rev_reg_def_json:Option< &'life5 str> ,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,'life4:'async_trait,'life5:'async_trait,Self:'async_trait;
            fn prover_delete_credential<'life0,'life1,'async_trait>(&'life0 self,cred_id: &'life1 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<()> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait;
            fn prover_create_link_secret<'life0,'life1,'async_trait>(&'life0 self,link_secret_id: &'life1 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait;
            fn issuer_create_schema<'life0,'life1,'life2,'life3,'life4,'async_trait>(&'life0 self,issuer_did: &'life1 str,name: &'life2 str,version: &'life3 str,attrs: &'life4 str,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<(String,String)> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,'life4:'async_trait,Self:'async_trait;
            fn revoke_credential_local<'life0,'life1,'life2,'life3,'async_trait>(&'life0 self,tails_dir: &'life1 str,rev_reg_id: &'life2 str,cred_rev_id: &'life3 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<()> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,Self:'async_trait;
            fn publish_local_revocations<'life0,'life1,'life2,'async_trait>(&'life0 self,submitter_did: &'life1 str,rev_reg_id: &'life2 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<()> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,Self:'async_trait;
            fn generate_nonce<'life0,'async_trait>(&'life0 self) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait;
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod unit_tests {

    use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
    use aries_vcx_core::errors::error::{AriesVcxCoreErrorKind, VcxCoreResult};

    use crate::utils::mockdata::profile::mock_anoncreds::MockAnoncreds;

    #[tokio::test]
    async fn test_unimplemented_methods() {
        // test used to assert which methods are unimplemented currently, can be removed after all methods implemented

        fn assert_unimplemented<T: std::fmt::Debug>(result: VcxCoreResult<T>) {
            assert_eq!(result.unwrap_err().kind(), AriesVcxCoreErrorKind::UnimplementedFeature)
        }

        let anoncreds: Box<dyn BaseAnonCreds> = Box::new(MockAnoncreds);

        assert_unimplemented(anoncreds.verifier_verify_proof("", "", "", "", "", "").await);
        assert_unimplemented(anoncreds.issuer_create_and_store_revoc_reg("", "", "", 0, "").await);
        assert_unimplemented(
            anoncreds
                .issuer_create_and_store_credential_def("", "", "", None, "")
                .await,
        );
        assert_unimplemented(anoncreds.prover_get_credential("").await);
        assert_unimplemented(anoncreds.prover_get_credentials(None).await);
        assert_unimplemented(anoncreds.prover_get_credentials_for_proof_req("").await);
        assert_unimplemented(anoncreds.prover_delete_credential("").await);
        assert_unimplemented(anoncreds.issuer_create_schema("", "", "", "").await);
    }
}
