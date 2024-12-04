use std::{fmt::Debug, sync::Arc};

use anoncreds_types::data_types::{
    identifiers::{
        cred_def_id::CredentialDefinitionId, issuer_id::IssuerId,
        rev_reg_def_id::RevocationRegistryDefinitionId, schema_id::SchemaId,
    },
    ledger::{
        cred_def::CredentialDefinition,
        rev_reg::RevocationRegistry,
        rev_reg_def::RevocationRegistryDefinition,
        rev_reg_delta::RevocationRegistryDelta,
        rev_status_list::RevocationStatusList,
        schema::{AttributeNames, Schema},
    },
};
use async_trait::async_trait;
use chrono::DateTime;
use did_cheqd::resolution::resolver::DidCheqdResolver;
use did_parser_nom::{Did, DidUrl};
use did_resolver::shared_types::did_resource::{DidResource, DidResourceMetadata};
use models::{
    CheqdAnoncredsCredentialDefinition, CheqdAnoncredsRevocationRegistryDefinition,
    CheqdAnoncredsRevocationStatusList, CheqdAnoncredsSchema,
};

use super::base_ledger::AnoncredsLedgerRead;
use crate::errors::error::{VcxLedgerError, VcxLedgerResult};

mod models;

const SCHEMA_RESOURCE_TYPE: &str = "anonCredsSchema";
const CRED_DEF_RESOURCE_TYPE: &str = "anonCredsCredDef";
const REV_REG_DEF_RESOURCE_TYPE: &str = "anonCredsRevocRegDef";
const STATUS_LIST_RESOURCE_TYPE: &str = "anonCredsStatusList";

pub struct CheqdAnoncredsLedgerRead {
    resolver: Arc<DidCheqdResolver>,
}

impl CheqdAnoncredsLedgerRead {
    pub fn new(resolver: Arc<DidCheqdResolver>) -> Self {
        Self { resolver }
    }

    fn check_resource_type(&self, resource: &DidResource, expected: &str) -> VcxLedgerResult<()> {
        let rtyp = &resource.metadata.resource_type;
        if rtyp != expected {
            return Err(VcxLedgerError::InvalidLedgerResponse(format!(
                "Returned resource is not expected type. Got {rtyp}, expected: {expected}"
            )));
        }
        Ok(())
    }

    async fn get_rev_reg_def_with_metadata(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
    ) -> VcxLedgerResult<(RevocationRegistryDefinition, DidResourceMetadata)> {
        let url = DidUrl::parse(rev_reg_id.to_string())?;
        let resource = self.resolver.resolve_resource(&url).await?;
        self.check_resource_type(&resource, REV_REG_DEF_RESOURCE_TYPE)?;

        let data: CheqdAnoncredsRevocationRegistryDefinition =
            serde_json::from_slice(&resource.content)?;
        Ok((
            RevocationRegistryDefinition {
                id: rev_reg_id.to_owned(),
                revoc_def_type: data.revoc_def_type,
                tag: data.tag,
                cred_def_id: data.cred_def_id,
                value: data.value,
                issuer_id: extract_issuer_id(&url)?,
            },
            resource.metadata,
        ))
    }
}

#[async_trait]
impl AnoncredsLedgerRead for CheqdAnoncredsLedgerRead {
    async fn get_schema(&self, schema_id: &SchemaId, _: Option<&Did>) -> VcxLedgerResult<Schema> {
        let url = DidUrl::parse(schema_id.to_string())?;
        let resource = self.resolver.resolve_resource(&url).await?;
        self.check_resource_type(&resource, SCHEMA_RESOURCE_TYPE)?;

        let data: CheqdAnoncredsSchema = serde_json::from_slice(&resource.content)?;
        Ok(Schema {
            id: schema_id.to_owned(),
            seq_no: None,
            name: data.name,
            version: data.version,
            attr_names: AttributeNames(data.attr_names),
            issuer_id: extract_issuer_id(&url)?,
        })
    }

    async fn get_cred_def(
        &self,
        cred_def_id: &CredentialDefinitionId,
        _: Option<&Did>,
    ) -> VcxLedgerResult<CredentialDefinition> {
        let url = DidUrl::parse(cred_def_id.to_string())?;
        let resource = self.resolver.resolve_resource(&url).await?;
        self.check_resource_type(&resource, CRED_DEF_RESOURCE_TYPE)?;

        let data: CheqdAnoncredsCredentialDefinition = serde_json::from_slice(&resource.content)?;
        Ok(CredentialDefinition {
            id: cred_def_id.to_owned(),
            schema_id: data.schema_id,
            signature_type: data.signature_type,
            tag: data.tag,
            value: data.value,
            issuer_id: extract_issuer_id(&url)?,
        })
    }

    async fn get_rev_reg_def_json(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
    ) -> VcxLedgerResult<RevocationRegistryDefinition> {
        self.get_rev_reg_def_with_metadata(rev_reg_id)
            .await
            .map(|v| v.0)
    }

    async fn get_rev_reg_delta_json(
        &self,
        _rev_reg_id: &RevocationRegistryDefinitionId,
        _from: Option<u64>,
        _to: Option<u64>,
    ) -> VcxLedgerResult<(RevocationRegistryDelta, u64)> {
        // unsupported, to be removed: https://github.com/hyperledger/aries-vcx/issues/1309
        Err(VcxLedgerError::UnimplementedFeature(
            "get_rev_reg_delta_json not supported for cheqd".into(),
        ))
    }

    async fn get_rev_status_list(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        timestamp: u64,
        // unused, we need to fetch anyway for resourceName
        _pre_fetched_rev_reg_def: Option<&RevocationRegistryDefinition>,
    ) -> VcxLedgerResult<(RevocationStatusList, u64)> {
        let rev_reg_def_url = DidUrl::parse(rev_reg_id.to_string())?;

        let (_def, rev_reg_def_metadata) = self.get_rev_reg_def_with_metadata(rev_reg_id).await?;

        //credo-ts uses the metadata.name or fails (https://docs.cheqd.io/product/advanced/anoncreds/revocation-status-list#same-resource-name-different-resource-type)
        let name = &rev_reg_def_metadata.resource_name;

        let did = rev_reg_def_url
            .did()
            .ok_or(VcxLedgerError::InvalidInput(format!(
                "DID URL missing DID {rev_reg_def_url}"
            )))?;

        let resource_dt =
            DateTime::from_timestamp(timestamp as i64, 0).ok_or(VcxLedgerError::InvalidInput(
                format!("input status list timestamp is not valid {timestamp}"),
            ))?;
        let xml_dt = resource_dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let query = format!(
            "{did}?resourceType={STATUS_LIST_RESOURCE_TYPE}&resourceName={name}&\
             resourceVersionTime={xml_dt}"
        );
        let query_url = DidUrl::parse(query)?;

        let resource = self.resolver.resolve_resource(&query_url).await?;
        self.check_resource_type(&resource, STATUS_LIST_RESOURCE_TYPE)?;

        let data: CheqdAnoncredsRevocationStatusList = serde_json::from_slice(&resource.content)?;
        let timestamp = resource.metadata.created.timestamp() as u64;

        let status_list = RevocationStatusList {
            rev_reg_def_id: Some(rev_reg_id.to_owned()),
            issuer_id: extract_issuer_id(&rev_reg_def_url)?,
            revocation_list: data.revocation_list,
            accum: data.accum,
            timestamp: Some(timestamp),
        };

        Ok((status_list, timestamp))
    }

    async fn get_rev_reg(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        timestamp: u64,
    ) -> VcxLedgerResult<(RevocationRegistry, u64)> {
        let (list, last_updated) = self
            .get_rev_status_list(rev_reg_id, timestamp, None)
            .await?;

        let accum = list
            .accum
            .ok_or(VcxLedgerError::InvalidLedgerResponse(format!(
                "response status list is missing an accumulator: {list:?}"
            )))?;

        let reg = RevocationRegistry {
            value: accum.into(),
        };

        Ok((reg, last_updated))
    }
}

fn extract_issuer_id(url: &DidUrl) -> VcxLedgerResult<IssuerId> {
    let did = url.did().ok_or(VcxLedgerError::InvalidInput(format!(
        "DID URL is missing a DID: {url}"
    )))?;
    IssuerId::new(did)
        .map_err(|e| VcxLedgerError::InvalidInput(format!("DID is not an IssuerId {e}")))
}

impl Debug for CheqdAnoncredsLedgerRead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CheqdAnoncredsLedgerRead instance")
    }
}
