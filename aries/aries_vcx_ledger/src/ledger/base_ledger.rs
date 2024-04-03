use std::fmt::Debug;

use anoncreds_types::data_types::{
    identifiers::{
        cred_def_id::CredentialDefinitionId, rev_reg_def_id::RevocationRegistryDefinitionId,
        schema_id::SchemaId,
    },
    ledger::{
        cred_def::CredentialDefinition, rev_reg::RevocationRegistry,
        rev_reg_def::RevocationRegistryDefinition, rev_reg_delta::RevocationRegistryDelta,
        schema::Schema,
    },
};
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use async_trait::async_trait;
use did_parser_nom::Did;
use indy_vdr::ledger::constants::UpdateRole;
use public_key::Key;
use serde::Serialize;

use crate::errors::error::VcxLedgerResult;

#[async_trait]
pub trait IndyLedgerRead: Debug + Send + Sync {
    async fn get_attr(&self, target_did: &Did, attr_name: &str) -> VcxLedgerResult<String>;
    async fn get_nym(&self, did: &Did) -> VcxLedgerResult<String>;
    async fn get_txn_author_agreement(&self) -> VcxLedgerResult<Option<String>>;
    async fn get_ledger_txn(
        &self,
        seq_no: i32,
        submitter_did: Option<&Did>,
    ) -> VcxLedgerResult<String>;
}

#[async_trait]
pub trait IndyLedgerWrite: Debug + Send + Sync {
    async fn publish_nym(
        &self,
        wallet: &impl BaseWallet,
        submitter_did: &Did,
        target_did: &Did,
        verkey: Option<&Key>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxLedgerResult<String>;
    async fn set_endorser(
        &self,
        wallet: &impl BaseWallet,
        submitter_did: &Did,
        request: &str,
        endorser: &Did,
    ) -> VcxLedgerResult<String>;
    async fn endorse_transaction(
        &self,
        wallet: &impl BaseWallet,
        endorser_did: &Did,
        request_json: &str,
    ) -> VcxLedgerResult<()>;
    async fn add_attr(
        &self,
        wallet: &impl BaseWallet,
        target_did: &Did,
        attrib_json: &str,
    ) -> VcxLedgerResult<String>;
    async fn write_did(
        &self,
        wallet: &impl BaseWallet,
        submitter_did: &Did,
        target_did: &Did,
        target_vk: &Key,
        role: Option<UpdateRole>,
        alias: Option<String>,
    ) -> VcxLedgerResult<String>;
}

#[async_trait]
pub trait AnoncredsLedgerRead: Debug + Send + Sync {
    async fn get_schema(
        &self,
        schema_id: &SchemaId,
        submitter_did: Option<&Did>,
    ) -> VcxLedgerResult<Schema>;
    async fn get_cred_def(
        &self,
        cred_def_id: &CredentialDefinitionId,
        submitter_did: Option<&Did>,
    ) -> VcxLedgerResult<CredentialDefinition>;
    async fn get_rev_reg_def_json(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
    ) -> VcxLedgerResult<RevocationRegistryDefinition>;
    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxLedgerResult<(RevocationRegistryDelta, u64)>;
    async fn get_rev_reg(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        timestamp: u64,
    ) -> VcxLedgerResult<(RevocationRegistry, u64)>;
}

#[async_trait]
pub trait AnoncredsLedgerWrite: Debug + Send + Sync {
    async fn publish_schema(
        &self,
        wallet: &impl BaseWallet,
        schema_json: Schema,
        submitter_did: &Did,
        endorser_did: Option<&Did>,
    ) -> VcxLedgerResult<()>;
    async fn publish_cred_def(
        &self,
        wallet: &impl BaseWallet,
        cred_def_json: CredentialDefinition,
        submitter_did: &Did,
    ) -> VcxLedgerResult<()>;
    async fn publish_rev_reg_def(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_def: RevocationRegistryDefinition,
        submitter_did: &Did,
    ) -> VcxLedgerResult<()>;
    async fn publish_rev_reg_delta(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_id: &RevocationRegistryDefinitionId,
        rev_reg_entry_json: RevocationRegistryDelta,
        submitter_did: &Did,
    ) -> VcxLedgerResult<()>;
}

pub trait TaaConfigurator: Debug + Send + Sync {
    fn set_txn_author_agreement_options(
        &self,
        taa_options: TxnAuthrAgrmtOptions,
    ) -> VcxLedgerResult<()>;
    fn get_txn_author_agreement_options(&self) -> VcxLedgerResult<Option<TxnAuthrAgrmtOptions>>;
}

#[derive(Clone, Serialize)]
pub struct TxnAuthrAgrmtOptions {
    pub text: String,
    pub version: String,
    pub mechanism: String,
}
