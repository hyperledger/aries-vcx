use anoncreds_types::data_types::{
    identifiers::{
        cred_def_id::CredentialDefinitionId, rev_reg_def_id::RevocationRegistryDefinitionId,
        schema_id::SchemaId,
    },
    ledger::{
        cred_def::CredentialDefinition, rev_reg::RevocationRegistry,
        rev_reg_def::RevocationRegistryDefinition, rev_reg_delta::RevocationRegistryDelta,
        rev_status_list::RevocationStatusList, schema::Schema,
    },
};
use aries_vcx_ledger::{
    errors::error::{VcxLedgerError, VcxLedgerResult},
    ledger::{
        base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite},
        indy_vdr_ledger::UpdateRole,
    },
};
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use async_trait::async_trait;
use did_parser_nom::Did;
use public_key::Key;

use crate::constants::{
    rev_def_json, CRED_DEF_JSON, DEFAULT_AUTHOR_AGREEMENT, REQUEST_WITH_ENDORSER,
    REV_REG_DELTA_JSON, REV_REG_JSON, REV_STATUS_LIST_JSON, SCHEMA_JSON,
};

#[derive(Debug)]
pub struct MockLedger;

#[allow(unused)]
#[async_trait]
impl IndyLedgerRead for MockLedger {
    async fn get_txn_author_agreement(&self) -> VcxLedgerResult<Option<String>> {
        Ok(Some(DEFAULT_AUTHOR_AGREEMENT.to_string()))
    }

    async fn get_nym(&self, did: &Did) -> VcxLedgerResult<String> {
        // not needed yet
        Err(VcxLedgerError::UnimplementedFeature(
            "unimplemented mock method: get_nym".into(),
        ))
    }

    async fn get_attr(&self, target_did: &Did, attr_name: &str) -> VcxLedgerResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }

    async fn get_ledger_txn(
        &self,
        seq_no: i32,
        submitter_did: Option<&Did>,
    ) -> VcxLedgerResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }
}

#[allow(unused)]
#[async_trait]
impl IndyLedgerWrite for MockLedger {
    async fn set_endorser(
        &self,
        wallet: &impl BaseWallet,
        submitter_did: &Did,
        request: &str,
        endorser: &Did,
    ) -> VcxLedgerResult<String> {
        Ok(REQUEST_WITH_ENDORSER.to_string())
    }

    async fn endorse_transaction(
        &self,
        wallet: &impl BaseWallet,
        endorser_did: &Did,
        request_json: &str,
    ) -> VcxLedgerResult<()> {
        Ok(())
    }

    async fn publish_nym(
        &self,
        wallet: &impl BaseWallet,
        submitter_did: &Did,
        target_did: &Did,
        verkey: Option<&Key>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxLedgerResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }

    async fn add_attr(
        &self,
        wallet: &impl BaseWallet,
        target_did: &Did,
        attrib_json: &str,
    ) -> VcxLedgerResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }

    async fn write_did(
        &self,
        wallet: &impl BaseWallet,
        submitter_did: &Did,
        target_did: &Did,
        target_vk: &Key,
        role: Option<UpdateRole>,
        alias: Option<String>,
    ) -> VcxLedgerResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }
}

#[allow(unused)]
#[async_trait]
impl AnoncredsLedgerRead for MockLedger {
    type RevocationRegistryDefinitionAdditionalMetadata = ();

    async fn get_schema(
        &self,
        schema_id: &SchemaId,
        submitter_did: Option<&Did>,
    ) -> VcxLedgerResult<Schema> {
        Ok(serde_json::from_str(SCHEMA_JSON)?)
    }

    async fn get_cred_def(
        &self,
        cred_def_id: &CredentialDefinitionId,
        submitter_did: Option<&Did>,
    ) -> VcxLedgerResult<CredentialDefinition> {
        Ok(serde_json::from_str(CRED_DEF_JSON)?)
    }

    async fn get_rev_reg_def_json(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
    ) -> VcxLedgerResult<(RevocationRegistryDefinition, ())> {
        Ok((rev_def_json(), ()))
    }

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxLedgerResult<(RevocationRegistryDelta, u64)> {
        Ok((serde_json::from_str(REV_REG_DELTA_JSON).unwrap(), 1))
    }

    async fn get_rev_status_list(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        timestamp: u64,
        meta: Option<&()>,
    ) -> VcxLedgerResult<(RevocationStatusList, u64)> {
        Ok((serde_json::from_str(REV_STATUS_LIST_JSON).unwrap(), 1))
    }

    async fn get_rev_reg(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        timestamp: u64,
    ) -> VcxLedgerResult<(RevocationRegistry, u64)> {
        Ok((serde_json::from_str(REV_REG_JSON).unwrap(), 1))
    }
}

#[allow(unused)]
#[async_trait]
impl AnoncredsLedgerWrite for MockLedger {
    async fn publish_schema(
        &self,
        wallet: &impl BaseWallet,
        schema_json: Schema,
        submitter_did: &Did,
        endorser_did: Option<&Did>,
    ) -> VcxLedgerResult<()> {
        Ok(())
    }

    async fn publish_cred_def(
        &self,
        wallet: &impl BaseWallet,
        cred_def_json: CredentialDefinition,
        submitter_did: &Did,
    ) -> VcxLedgerResult<()> {
        Ok(())
    }

    async fn publish_rev_reg_def(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_def: RevocationRegistryDefinition,
        submitter_did: &Did,
    ) -> VcxLedgerResult<()> {
        Ok(())
    }

    async fn publish_rev_reg_delta(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_id: &RevocationRegistryDefinitionId,
        rev_reg_entry_json: RevocationRegistryDelta,
        submitter_did: &Did,
    ) -> VcxLedgerResult<()> {
        Ok(())
    }
}
