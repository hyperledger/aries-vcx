//! Contains convenience wrappers for the aries_vcx_ledger traits when working with [Arc]s.
use std::sync::Arc;

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
use async_trait::async_trait;
use did_parser_nom::Did;

use super::base_ledger::AnoncredsLedgerRead;
use crate::errors::error::VcxLedgerResult;

/// Convenience trait to convert something into an [AnoncredsLedgerRead] implementation.
pub trait IntoAnoncredsLedgerRead {
    fn into_impl(self) -> impl AnoncredsLedgerRead;
}

/// Convenience to convert any Arc<AnoncredsLedgerRead> into AnoncredsLedgerRead.
/// This is possible because all methods of [AnoncredsLedgerRead] only require a reference
/// of self.
impl<T> IntoAnoncredsLedgerRead for Arc<T>
where
    T: AnoncredsLedgerRead,
{
    fn into_impl(self) -> impl AnoncredsLedgerRead {
        ArcAnoncredsLedgerRead(self)
    }
}

#[derive(Debug)]
struct ArcAnoncredsLedgerRead<T: AnoncredsLedgerRead>(Arc<T>);

#[async_trait]
impl<T> AnoncredsLedgerRead for ArcAnoncredsLedgerRead<T>
where
    T: AnoncredsLedgerRead,
{
    type RevocationRegistryDefinitionAdditionalMetadata =
        T::RevocationRegistryDefinitionAdditionalMetadata;

    async fn get_schema(
        &self,
        schema_id: &SchemaId,
        submitter_did: Option<&Did>,
    ) -> VcxLedgerResult<Schema> {
        self.0.get_schema(schema_id, submitter_did).await
    }

    async fn get_cred_def(
        &self,
        cred_def_id: &CredentialDefinitionId,
        submitter_did: Option<&Did>,
    ) -> VcxLedgerResult<CredentialDefinition> {
        self.get_cred_def(cred_def_id, submitter_did).await
    }
    async fn get_rev_reg_def_json(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
    ) -> VcxLedgerResult<(
        RevocationRegistryDefinition,
        Self::RevocationRegistryDefinitionAdditionalMetadata,
    )> {
        self.get_rev_reg_def_json(rev_reg_id).await
    }

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxLedgerResult<(RevocationRegistryDelta, u64)> {
        #[allow(deprecated)]
        self.get_rev_reg_delta_json(rev_reg_id, from, to).await
    }

    async fn get_rev_status_list(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        timestamp: u64,
        rev_reg_def_meta: Option<&Self::RevocationRegistryDefinitionAdditionalMetadata>,
    ) -> VcxLedgerResult<(RevocationStatusList, u64)> {
        self.get_rev_status_list(rev_reg_id, timestamp, rev_reg_def_meta)
            .await
    }
    async fn get_rev_reg(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        timestamp: u64,
    ) -> VcxLedgerResult<(RevocationRegistry, u64)> {
        self.get_rev_reg(rev_reg_id, timestamp).await
    }
}
