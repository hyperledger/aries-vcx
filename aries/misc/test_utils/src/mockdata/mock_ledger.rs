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
use aries_vcx_core::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
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
    REV_REG_DELTA_JSON, REV_REG_JSON, SCHEMA_JSON,
};

#[derive(Debug)]
pub struct MockLedger;

#[allow(unused)]
#[async_trait]
impl IndyLedgerRead for MockLedger {
    async fn get_txn_author_agreement(&self) -> VcxCoreResult<Option<String>> {
        Ok(Some(DEFAULT_AUTHOR_AGREEMENT.to_string()))
    }

    async fn get_nym(&self, did: &Did) -> VcxCoreResult<String> {
        // not needed yet
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "unimplemented mock method: get_nym",
        ))
    }

    async fn get_attr(&self, target_did: &Did, attr_name: &str) -> VcxCoreResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }

    async fn get_ledger_txn(
        &self,
        seq_no: i32,
        submitter_did: Option<&Did>,
    ) -> VcxCoreResult<String> {
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
    ) -> VcxCoreResult<String> {
        Ok(REQUEST_WITH_ENDORSER.to_string())
    }

    async fn endorse_transaction(
        &self,
        wallet: &impl BaseWallet,
        endorser_did: &Did,
        request_json: &str,
    ) -> VcxCoreResult<()> {
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
    ) -> VcxCoreResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }

    async fn add_attr(
        &self,
        wallet: &impl BaseWallet,
        target_did: &Did,
        attrib_json: &str,
    ) -> VcxCoreResult<String> {
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
    ) -> VcxCoreResult<String> {
        Ok(r#"{"rc":"success"}"#.to_string())
    }
}

#[allow(unused)]
#[async_trait]
impl AnoncredsLedgerRead for MockLedger {
    async fn get_schema(
        &self,
        schema_id: &SchemaId,
        submitter_did: Option<&Did>,
    ) -> VcxCoreResult<Schema> {
        Ok(serde_json::from_str(SCHEMA_JSON)?)
    }

    async fn get_cred_def(
        &self,
        cred_def_id: &CredentialDefinitionId,
        submitter_did: Option<&Did>,
    ) -> VcxCoreResult<CredentialDefinition> {
        Ok(serde_json::from_str(CRED_DEF_JSON)?)
    }

    async fn get_rev_reg_def_json(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
    ) -> VcxCoreResult<RevocationRegistryDefinition> {
        Ok(rev_def_json())
    }

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxCoreResult<(RevocationRegistryDelta, u64)> {
        Ok((serde_json::from_str(REV_REG_DELTA_JSON).unwrap(), 1))
    }

    async fn get_rev_reg(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        timestamp: u64,
    ) -> VcxCoreResult<(RevocationRegistry, u64)> {
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
    ) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn publish_cred_def(
        &self,
        wallet: &impl BaseWallet,
        cred_def_json: CredentialDefinition,
        submitter_did: &Did,
    ) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn publish_rev_reg_def(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_def: RevocationRegistryDefinition,
        submitter_did: &Did,
    ) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn publish_rev_reg_delta(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_id: &RevocationRegistryDefinitionId,
        rev_reg_entry_json: RevocationRegistryDelta,
        submitter_did: &Did,
    ) -> VcxCoreResult<()> {
        Ok(())
    }
}
