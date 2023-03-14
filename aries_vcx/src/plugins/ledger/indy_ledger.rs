use std::sync::Arc;

use async_trait::async_trait;

use super::base_ledger::BaseLedger;
use crate::{
    common::primitives::revocation_registry::RevocationRegistryDefinition, core::profile::indy_profile::IndySdkProfile,
    errors::error::VcxResult, indy,
};

pub struct IndySdkLedger {
    profile: Arc<IndySdkProfile>,
}

impl IndySdkLedger {
    pub fn new(profile: Arc<IndySdkProfile>) -> Self {
        IndySdkLedger { profile }
    }
}

#[async_trait]
impl BaseLedger for IndySdkLedger {
    async fn sign_and_submit_request(&self, submitter_did: &str, request_json: &str) -> VcxResult<String> {
        indy::ledger::transactions::libindy_sign_and_submit_request(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            submitter_did,
            request_json,
        )
        .await
    }

    async fn submit_request(&self, request_json: &str) -> VcxResult<String> {
        indy::ledger::transactions::libindy_submit_request(self.profile.indy_pool_handle, request_json).await
    }

    async fn endorse_transaction(&self, endorser_did: &str, request_json: &str) -> VcxResult<()> {
        indy::ledger::transactions::endorse_transaction(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            endorser_did,
            request_json,
        )
        .await
    }

    async fn set_endorser(&self, submitter_did: &str, request_json: &str, endorser: &str) -> VcxResult<String> {
        indy::ledger::transactions::set_endorser(self.profile.indy_wallet_handle, submitter_did, request_json, endorser)
            .await
    }

    async fn get_txn_author_agreement(&self) -> VcxResult<String> {
        indy::ledger::transactions::libindy_get_txn_author_agreement(self.profile.indy_pool_handle).await
    }

    async fn get_nym(&self, did: &str) -> VcxResult<String> {
        indy::ledger::transactions::get_nym(self.profile.indy_pool_handle, did).await
    }

    // returns request result as JSON
    async fn publish_nym(
        &self,
        submitter_did: &str,
        target_did: &str,
        verkey: Option<&str>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxResult<String> {
        let nym_request =
            indy::ledger::transactions::libindy_build_nym_request(submitter_did, target_did, verkey, data, role)
                .await?;
        let nym_request = indy::ledger::transactions::append_txn_author_agreement_to_request(&nym_request).await?;

        indy::ledger::transactions::libindy_sign_and_submit_request(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            submitter_did,
            &nym_request,
        )
        .await
    }

    async fn get_schema(&self, schema_id: &str, submitter_did: Option<&str>) -> VcxResult<String> {
        if let Some(submitter_did) = submitter_did {
            // with cache if possible
            indy::ledger::transactions::libindy_get_schema(
                self.profile.indy_wallet_handle,
                self.profile.indy_pool_handle,
                submitter_did,
                schema_id,
            )
            .await
        } else {
            // no cache
            indy::ledger::transactions::get_schema_json(
                self.profile.indy_wallet_handle,
                self.profile.indy_pool_handle,
                schema_id,
            )
            .await
            .map(|(_, json)| json)
        }
    }

    async fn get_cred_def(&self, cred_def_id: &str, _submitter_did: Option<&str>) -> VcxResult<String> {
        indy::ledger::transactions::get_cred_def_json(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            cred_def_id,
        )
        .await
        .map(|(_id, json)| json)
    }

    async fn get_attr(&self, target_did: &str, attr_name: &str) -> VcxResult<String> {
        indy::ledger::transactions::get_attr(self.profile.indy_pool_handle, target_did, attr_name).await
    }

    async fn add_attr(&self, target_did: &str, attrib_json: &str) -> VcxResult<String> {
        indy::ledger::transactions::add_attr(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            target_did,
            attrib_json,
        )
        .await
    }

    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxResult<String> {
        indy::ledger::transactions::get_rev_reg_def_json(self.profile.indy_pool_handle, rev_reg_id)
            .await
            .map(|(_, json)| json)
    }

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxResult<(String, String, u64)> {
        indy::ledger::transactions::get_rev_reg_delta_json(self.profile.indy_pool_handle, rev_reg_id, from, to).await
    }

    async fn get_rev_reg(&self, rev_reg_id: &str, timestamp: u64) -> VcxResult<(String, String, u64)> {
        indy::ledger::transactions::get_rev_reg(self.profile.indy_pool_handle, rev_reg_id, timestamp).await
    }

    async fn get_ledger_txn(&self, seq_no: i32, submitter_did: Option<&str>) -> VcxResult<String> {
        indy::ledger::transactions::get_ledger_txn(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            seq_no,
            submitter_did,
        )
        .await
    }

    async fn build_schema_request(&self, submitter_did: &str, schema_json: &str) -> VcxResult<String> {
        indy::ledger::transactions::build_schema_request(submitter_did, schema_json).await
    }

    async fn publish_schema(
        &self,
        schema_json: &str,
        submitter_did: &str,
        endorser_did: Option<String>,
    ) -> VcxResult<()> {
        indy::primitives::credential_schema::publish_schema(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            submitter_did,
            schema_json,
            endorser_did,
        )
        .await
    }

    async fn publish_cred_def(&self, cred_def_json: &str, submitter_did: &str) -> VcxResult<()> {
        indy::primitives::credential_definition::publish_cred_def(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            submitter_did,
            cred_def_json,
        )
        .await
    }

    async fn publish_rev_reg_def(
        &self,
        rev_reg_def: &RevocationRegistryDefinition,
        submitter_did: &str,
    ) -> VcxResult<()> {
        indy::primitives::revocation_registry::publish_rev_reg_def(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            submitter_did,
            rev_reg_def,
        )
        .await
    }

    async fn publish_rev_reg_delta(
        &self,
        rev_reg_id: &str,
        rev_reg_entry_json: &str,
        submitter_did: &str,
    ) -> VcxResult<()> {
        indy::primitives::revocation_registry::publish_rev_reg_delta(
            self.profile.indy_wallet_handle,
            self.profile.indy_pool_handle,
            submitter_did,
            rev_reg_id,
            rev_reg_entry_json,
        )
        .await?;

        Ok(())
    }
}
