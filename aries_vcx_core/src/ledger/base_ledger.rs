use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use indy_vdr::ledger::constants::UpdateRole;
use serde::Serialize;

use crate::errors::error::VcxCoreResult;

#[async_trait]
pub trait IndyLedgerRead: Debug + Send + Sync {
    async fn get_attr(&self, target_did: &str, attr_name: &str) -> VcxCoreResult<String>;
    async fn get_nym(&self, did: &str) -> VcxCoreResult<String>;
    async fn get_txn_author_agreement(&self) -> VcxCoreResult<Option<String>>;
    async fn get_ledger_txn(
        &self,
        seq_no: i32,
        submitter_did: Option<&str>,
    ) -> VcxCoreResult<String>;
}

#[async_trait]
pub trait IndyLedgerWrite: Debug + Send + Sync {
    async fn publish_nym(
        &self,
        submitter_did: &str,
        target_did: &str,
        verkey: Option<&str>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxCoreResult<String>;
    async fn set_endorser(
        &self,
        submitter_did: &str,
        request: &str,
        endorser: &str,
    ) -> VcxCoreResult<String>;
    async fn endorse_transaction(
        &self,
        endorser_did: &str,
        request_json: &str,
    ) -> VcxCoreResult<()>;
    async fn add_attr(&self, target_did: &str, attrib_json: &str) -> VcxCoreResult<String>;
    async fn write_did(
        &self,
        submitter_did: &str,
        target_did: &str,
        target_vk: &str,
        role: Option<UpdateRole>,
        alias: Option<String>,
    ) -> VcxCoreResult<String>;
}

#[async_trait]
pub trait AnoncredsLedgerRead: Debug + Send + Sync {
    async fn get_schema(
        &self,
        schema_id: &str,
        submitter_did: Option<&str>,
    ) -> VcxCoreResult<String>;
    async fn get_cred_def(
        &self,
        cred_def_id: &str,
        submitter_did: Option<&str>,
    ) -> VcxCoreResult<String>;
    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxCoreResult<String>;
    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxCoreResult<(String, String, u64)>;
    async fn get_rev_reg(
        &self,
        rev_reg_id: &str,
        timestamp: u64,
    ) -> VcxCoreResult<(String, String, u64)>;
}

#[async_trait]
pub trait AnoncredsLedgerWrite: Debug + Send + Sync {
    async fn publish_schema(
        &self,
        schema_json: &str,
        submitter_did: &str,
        endorser_did: Option<String>,
    ) -> VcxCoreResult<()>;
    async fn publish_cred_def(&self, cred_def_json: &str, submitter_did: &str)
        -> VcxCoreResult<()>;
    async fn publish_rev_reg_def(
        &self,
        rev_reg_def: &str,
        submitter_did: &str,
    ) -> VcxCoreResult<()>;
    async fn publish_rev_reg_delta(
        &self,
        rev_reg_id: &str,
        rev_reg_entry_json: &str,
        submitter_did: &str,
    ) -> VcxCoreResult<()>;
}

pub trait TaaConfigurator: Debug + Send + Sync {
    fn set_txn_author_agreement_options(
        &self,
        taa_options: TxnAuthrAgrmtOptions,
    ) -> VcxCoreResult<()>;
    fn get_txn_author_agreement_options(&self) -> VcxCoreResult<Option<TxnAuthrAgrmtOptions>>;
}

#[derive(Clone, Serialize)]
pub struct TxnAuthrAgrmtOptions {
    pub text: String,
    pub version: String,
    pub mechanism: String,
}
/// TODO: Workaround to keep some bits and pieces (like libvcx) working with Arcs. Delete ASAP and
/// wash your hands after.
#[async_trait]
impl<T> IndyLedgerRead for Arc<T>
where
    T: IndyLedgerRead + ?Sized,
{
    async fn get_attr(&self, target_did: &str, attr_name: &str) -> VcxCoreResult<String> {
        (**self).get_attr(target_did, attr_name).await
    }
    async fn get_nym(&self, did: &str) -> VcxCoreResult<String> {
        (**self).get_nym(did).await
    }
    async fn get_txn_author_agreement(&self) -> VcxCoreResult<Option<String>> {
        (**self).get_txn_author_agreement().await
    }
    async fn get_ledger_txn(
        &self,
        seq_no: i32,
        submitter_did: Option<&str>,
    ) -> VcxCoreResult<String> {
        (**self).get_ledger_txn(seq_no, submitter_did).await
    }
}
/// TODO: Workaround to keep some bits and pieces (like libvcx) working with Arcs. Delete ASAP and
/// wash your hands after.
#[async_trait]
impl<T> IndyLedgerWrite for Arc<T>
where
    T: IndyLedgerWrite + ?Sized,
{
    async fn publish_nym(
        &self,
        submitter_did: &str,
        target_did: &str,
        verkey: Option<&str>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxCoreResult<String> {
        (**self)
            .publish_nym(submitter_did, target_did, verkey, data, role)
            .await
    }
    async fn set_endorser(
        &self,
        submitter_did: &str,
        request: &str,
        endorser: &str,
    ) -> VcxCoreResult<String> {
        (**self)
            .set_endorser(submitter_did, request, endorser)
            .await
    }
    async fn endorse_transaction(
        &self,
        endorser_did: &str,
        request_json: &str,
    ) -> VcxCoreResult<()> {
        (**self)
            .endorse_transaction(endorser_did, request_json)
            .await
    }
    async fn add_attr(&self, target_did: &str, attrib_json: &str) -> VcxCoreResult<String> {
        (**self).add_attr(target_did, attrib_json).await
    }
    async fn write_did(
        &self,
        submitter_did: &str,
        target_did: &str,
        target_vk: &str,
        role: Option<UpdateRole>,
        alias: Option<String>,
    ) -> VcxCoreResult<String> {
        (**self)
            .write_did(submitter_did, target_did, target_vk, role, alias)
            .await
    }
}

/// TODO: Workaround to keep some bits and pieces (like libvcx) working with Arcs. Delete ASAP and
/// wash your hands after.
#[async_trait]
impl<T> AnoncredsLedgerRead for Arc<T>
where
    T: AnoncredsLedgerRead + ?Sized,
{
    async fn get_schema(
        &self,
        schema_id: &str,
        submitter_did: Option<&str>,
    ) -> VcxCoreResult<String> {
        (**self).get_schema(schema_id, submitter_did).await
    }
    async fn get_cred_def(
        &self,
        cred_def_id: &str,
        submitter_did: Option<&str>,
    ) -> VcxCoreResult<String> {
        (**self).get_cred_def(cred_def_id, submitter_did).await
    }
    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxCoreResult<String> {
        (**self).get_rev_reg_def_json(rev_reg_id).await
    }
    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxCoreResult<(String, String, u64)> {
        (**self).get_rev_reg_delta_json(rev_reg_id, from, to).await
    }
    async fn get_rev_reg(
        &self,
        rev_reg_id: &str,
        timestamp: u64,
    ) -> VcxCoreResult<(String, String, u64)> {
        (**self).get_rev_reg(rev_reg_id, timestamp).await
    }
}

/// TODO: Workaround to keep some bits and pieces (like libvcx) working with Arcs. Delete ASAP and
/// wash your hands after.
#[async_trait]
impl<T> AnoncredsLedgerWrite for Arc<T>
where
    T: AnoncredsLedgerWrite + ?Sized,
{
    async fn publish_schema(
        &self,
        schema_json: &str,
        submitter_did: &str,
        endorser_did: Option<String>,
    ) -> VcxCoreResult<()> {
        (**self)
            .publish_schema(schema_json, submitter_did, endorser_did)
            .await
    }
    async fn publish_cred_def(
        &self,
        cred_def_json: &str,
        submitter_did: &str,
    ) -> VcxCoreResult<()> {
        (**self)
            .publish_cred_def(cred_def_json, submitter_did)
            .await
    }
    async fn publish_rev_reg_def(
        &self,
        rev_reg_def: &str,
        submitter_did: &str,
    ) -> VcxCoreResult<()> {
        (**self)
            .publish_rev_reg_def(rev_reg_def, submitter_did)
            .await
    }
    async fn publish_rev_reg_delta(
        &self,
        rev_reg_id: &str,
        rev_reg_entry_json: &str,
        submitter_did: &str,
    ) -> VcxCoreResult<()> {
        (**self)
            .publish_rev_reg_delta(rev_reg_id, rev_reg_entry_json, submitter_did)
            .await
    }
}
