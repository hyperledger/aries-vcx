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

use super::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerSupport};
use crate::errors::error::VcxLedgerResult;

/// Trait designed to convert [Arc<T>] into [ArcLedgerTraitWrapper<T>], such that
/// [Arc<T>] can inherit any trait implementation of [ArcLedgerTraitWrapper]. (e.g.
/// [AnoncredsLedgerRead], [AnoncredsLedgerSupport]).
pub trait IntoArcLedgerTrait<T>
where
    Self: Sized,
{
    fn into_impl(self) -> ArcLedgerTraitWrapper<T>;
}

impl<T> IntoArcLedgerTrait<T> for Arc<T> {
    fn into_impl(self) -> ArcLedgerTraitWrapper<T> {
        ArcLedgerTraitWrapper(self)
    }
}

/// Thin wrapper over some [Arc<T>]. Designed to implement relevant aries_vcx_ledger
/// traits on behalf of [Arc<T>], if [T] implements those traits. Necessary since [Arc<T>]
/// would not inherit those implementations automatically.
#[derive(Debug)]
pub struct ArcLedgerTraitWrapper<T>(Arc<T>);

#[async_trait]
impl<T> AnoncredsLedgerRead for ArcLedgerTraitWrapper<T>
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
        self.0.get_cred_def(cred_def_id, submitter_did).await
    }
    async fn get_rev_reg_def_json(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
    ) -> VcxLedgerResult<(
        RevocationRegistryDefinition,
        Self::RevocationRegistryDefinitionAdditionalMetadata,
    )> {
        self.0.get_rev_reg_def_json(rev_reg_id).await
    }

    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxLedgerResult<(RevocationRegistryDelta, u64)> {
        #[allow(deprecated)]
        self.0.get_rev_reg_delta_json(rev_reg_id, from, to).await
    }

    async fn get_rev_status_list(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        timestamp: u64,
        rev_reg_def_meta: Option<&Self::RevocationRegistryDefinitionAdditionalMetadata>,
    ) -> VcxLedgerResult<(RevocationStatusList, u64)> {
        self.0
            .get_rev_status_list(rev_reg_id, timestamp, rev_reg_def_meta)
            .await
    }
    async fn get_rev_reg(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        timestamp: u64,
    ) -> VcxLedgerResult<(RevocationRegistry, u64)> {
        self.0.get_rev_reg(rev_reg_id, timestamp).await
    }
}

impl<T> AnoncredsLedgerSupport for ArcLedgerTraitWrapper<T>
where
    T: AnoncredsLedgerSupport,
{
    fn supports_schema(&self, id: &SchemaId) -> bool {
        self.0.supports_schema(id)
    }

    fn supports_credential_definition(&self, id: &CredentialDefinitionId) -> bool {
        self.0.supports_credential_definition(id)
    }

    fn supports_revocation_registry(&self, id: &RevocationRegistryDefinitionId) -> bool {
        self.0.supports_revocation_registry(id)
    }
}
