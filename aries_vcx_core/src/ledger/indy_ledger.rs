use async_trait::async_trait;

use crate::errors::error::VcxCoreResult;
use crate::{indy, PoolHandle, WalletHandle};
use crate::global::settings;
use crate::indy::ledger::transactions::{_check_schema_response, build_cred_def_request, build_rev_reg_delta_request, build_rev_reg_request, build_schema_request, check_response, set_endorser, sign_and_submit_to_ledger};

use super::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite};
use super::map_error_not_found_to_none;

#[derive(Debug)]
pub struct IndySdkLedgerRead {
    indy_wallet_handle: WalletHandle,
    indy_pool_handle: PoolHandle,
}

impl IndySdkLedgerRead {
    pub fn new(indy_wallet_handle: WalletHandle, indy_pool_handle: PoolHandle) -> Self {
        IndySdkLedgerRead {
            indy_wallet_handle,
            indy_pool_handle,
        }
    }
}

#[derive(Debug)]
pub struct IndySdkLedgerWrite {
    indy_wallet_handle: WalletHandle,
    indy_pool_handle: PoolHandle,
}

impl IndySdkLedgerWrite {
    pub fn new(indy_wallet_handle: WalletHandle, indy_pool_handle: PoolHandle) -> Self {
        IndySdkLedgerWrite {
            indy_wallet_handle,
            indy_pool_handle,
        }
    }
}

#[async_trait]
impl IndyLedgerRead for IndySdkLedgerRead {
    async fn get_attr(&self, target_did: &str, attr_name: &str) -> VcxCoreResult<String> {
        indy::ledger::transactions::get_attr(self.indy_pool_handle, target_did, attr_name).await
    }

    async fn get_nym(&self, did: &str) -> VcxCoreResult<String> {
        indy::ledger::transactions::get_nym(self.indy_pool_handle, did).await
    }

    async fn get_txn_author_agreement(&self) -> VcxCoreResult<Option<String>> {
        map_error_not_found_to_none(
            indy::ledger::transactions::libindy_get_txn_author_agreement(self.indy_pool_handle).await,
        )
    }

    async fn get_ledger_txn(&self, seq_no: i32, submitter_did: Option<&str>) -> VcxCoreResult<String> {
        indy::ledger::transactions::get_ledger_txn(
            self.indy_wallet_handle,
            self.indy_pool_handle,
            seq_no,
            submitter_did,
        )
        .await
    }
}

#[async_trait]
impl IndyLedgerWrite for IndySdkLedgerWrite {
    async fn publish_nym(
        &self,
        submitter_did: &str,
        target_did: &str,
        verkey: Option<&str>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxCoreResult<String> {
        let nym_request =
            indy::ledger::transactions::libindy_build_nym_request(submitter_did, target_did, verkey, data, role)
                .await?;
        let nym_request = indy::ledger::transactions::append_txn_author_agreement_to_request(&nym_request).await?;

        indy::ledger::transactions::libindy_sign_and_submit_request(
            self.indy_wallet_handle,
            self.indy_pool_handle,
            submitter_did,
            &nym_request,
        )
        .await
    }

    async fn set_endorser(&self, submitter_did: &str, request: &str, endorser: &str) -> VcxCoreResult<String> {
        indy::ledger::transactions::set_endorser(self.indy_wallet_handle, submitter_did, request, endorser).await
    }

    async fn endorse_transaction(&self, endorser_did: &str, request_json: &str) -> VcxCoreResult<()> {
        indy::ledger::transactions::endorse_transaction(
            self.indy_wallet_handle,
            self.indy_pool_handle,
            endorser_did,
            request_json,
        )
        .await
    }

    async fn add_attr(&self, target_did: &str, attrib_json: &str) -> VcxCoreResult<String> {
        indy::ledger::transactions::add_attr(self.indy_wallet_handle, self.indy_pool_handle, target_did, attrib_json)
            .await
    }
}

#[async_trait]
impl AnoncredsLedgerRead for IndySdkLedgerRead {
    async fn get_schema(&self, schema_id: &str, submitter_did: Option<&str>) -> VcxCoreResult<String> {
        if let Some(submitter_did) = submitter_did {
            // with cache if possible
            indy::ledger::transactions::libindy_get_schema(
                self.indy_wallet_handle,
                self.indy_pool_handle,
                submitter_did,
                schema_id,
            )
            .await
        } else {
            // no cache
            indy::ledger::transactions::get_schema_json(self.indy_wallet_handle, self.indy_pool_handle, schema_id)
                .await
                .map(|(_, json)| json)
        }
    }

    async fn get_cred_def(&self, cred_def_id: &str, _submitter_did: Option<&str>) -> VcxCoreResult<String> {
        indy::ledger::transactions::get_cred_def_json(self.indy_wallet_handle, self.indy_pool_handle, cred_def_id)
            .await
            .map(|(_id, json)| json)
    }

    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxCoreResult<String> {
        indy::ledger::transactions::get_rev_reg_def_json(self.indy_pool_handle, rev_reg_id)
            .await
            .map(|(_, json)| json)
    }

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxCoreResult<(String, String, u64)> {
        indy::ledger::transactions::get_rev_reg_delta_json(self.indy_pool_handle, rev_reg_id, from, to).await
    }

    async fn get_rev_reg(&self, rev_reg_id: &str, timestamp: u64) -> VcxCoreResult<(String, String, u64)> {
        indy::ledger::transactions::get_rev_reg(self.indy_pool_handle, rev_reg_id, timestamp).await
    }
}

#[async_trait]
impl AnoncredsLedgerWrite for IndySdkLedgerWrite {
    async fn publish_schema(
        &self,
        schema_json: &str,
        submitter_did: &str,
        endorser_did: Option<String>,
    ) -> VcxCoreResult<()> {
        trace!(
        "publish_schema >>> submitter_did: {:?}, schema_json: {:?}, endorser_did: {:?}",
        submitter_did,
        schema_json,
        endorser_did
    );

        if settings::indy_mocks_enabled() {
            debug!("publish_schema >>> mocked success");
            return Ok(());
        }

        let mut request = build_schema_request(submitter_did, schema_json).await?;
        if let Some(endorser_did) = endorser_did {
            request = set_endorser(self.indy_wallet_handle, submitter_did, &request, &endorser_did).await?;
        }
        let response = sign_and_submit_to_ledger(self.indy_wallet_handle, self.indy_pool_handle, submitter_did, &request).await?;
        _check_schema_response(&response)?;

        Ok(())
    }

    async fn publish_cred_def(&self, cred_def_json: &str, issuer_did: &str) -> VcxCoreResult<()> {
        trace!(
        "publish_cred_def >>> issuer_did: {}, cred_def_json: {}",
        issuer_did,
        cred_def_json
    );
        if settings::indy_mocks_enabled() {
            debug!("publish_cred_def >>> mocked success");
            return Ok(());
        }
        let cred_def_req = build_cred_def_request(issuer_did, cred_def_json).await?;
        let response = sign_and_submit_to_ledger(self.indy_wallet_handle, self.indy_pool_handle, issuer_did, &cred_def_req).await?;
        check_response(&response)
    }

    async fn publish_rev_reg_def(&self, rev_reg_def: &str, issuer_did: &str) -> VcxCoreResult<()> {
        trace!("publish_rev_reg_def >>> issuer_did: {}, rev_reg_def: ...", issuer_did);
        if settings::indy_mocks_enabled() {
            debug!("publish_rev_reg_def >>> mocked success");
            return Ok(());
        }

        let rev_reg_def_req = build_rev_reg_request(issuer_did, rev_reg_def).await?;
        let response = sign_and_submit_to_ledger(self.indy_wallet_handle, self.indy_pool_handle, issuer_did, &rev_reg_def_req).await?;
        check_response(&response)
    }

    async fn publish_rev_reg_delta(
        &self,
        rev_reg_id: &str,
        revoc_reg_delta_json: &str,
        issuer_did: &str,
    ) -> VcxCoreResult<()> {
        trace!(
        "publish_rev_reg_delta >>> issuer_did: {}, rev_reg_id: {}, revoc_reg_delta_json: {}",
        issuer_did,
        rev_reg_id,
        revoc_reg_delta_json
    );

        let request = build_rev_reg_delta_request(issuer_did, rev_reg_id, revoc_reg_delta_json).await?;
        let response = sign_and_submit_to_ledger(self.indy_wallet_handle, self.indy_pool_handle, issuer_did, &request).await?;
        check_response(&response)?;

        Ok(())
    }
}
