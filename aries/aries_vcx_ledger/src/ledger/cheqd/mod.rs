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
use serde::{Deserialize, Serialize};
use url::Url;

use super::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerSupport};
use crate::errors::error::{VcxLedgerError, VcxLedgerResult};

mod models;

const SCHEMA_RESOURCE_TYPE: &str = "anonCredsSchema";
const CRED_DEF_RESOURCE_TYPE: &str = "anonCredsCredDef";
const REV_REG_DEF_RESOURCE_TYPE: &str = "anonCredsRevocRegDef";
const STATUS_LIST_RESOURCE_TYPE: &str = "anonCredsStatusList";

/// Struct for resolving anoncreds objects from cheqd ledgers using the cheqd
/// anoncreds object method: https://docs.cheqd.io/product/advanced/anoncreds.
///
/// Relies on a cheqd DID resolver ([DidCheqdResolver]) to fetch DID resources.
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

    async fn get_rev_reg_def_with_resource_metadata(
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevocationRegistryDefinitionAdditionalMetadata {
    pub resource_name: String,
}

#[async_trait]
impl AnoncredsLedgerRead for CheqdAnoncredsLedgerRead {
    type RevocationRegistryDefinitionAdditionalMetadata =
        RevocationRegistryDefinitionAdditionalMetadata;

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
    ) -> VcxLedgerResult<(
        RevocationRegistryDefinition,
        RevocationRegistryDefinitionAdditionalMetadata,
    )> {
        let (rev_reg_def, resource_meta) = self
            .get_rev_reg_def_with_resource_metadata(rev_reg_id)
            .await?;

        let meta = RevocationRegistryDefinitionAdditionalMetadata {
            resource_name: resource_meta.resource_name,
        };

        Ok((rev_reg_def, meta))
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
        rev_reg_def_meta: Option<&RevocationRegistryDefinitionAdditionalMetadata>,
    ) -> VcxLedgerResult<(RevocationStatusList, u64)> {
        let rev_reg_def_url = DidUrl::parse(rev_reg_id.to_string())?;

        // refetch if needed
        let rev_reg_def_meta = match rev_reg_def_meta {
            Some(v) => v,
            None => &self.get_rev_reg_def_json(rev_reg_id).await?.1,
        };

        //credo-ts uses the metadata.name or fails (https://docs.cheqd.io/product/advanced/anoncreds/revocation-status-list#same-resource-name-different-resource-type)
        let name = &rev_reg_def_meta.resource_name;

        let did = rev_reg_def_url
            .did()
            .ok_or(VcxLedgerError::InvalidInput(format!(
                "DID URL missing DID {rev_reg_def_url}"
            )))?;

        let resource_dt =
            DateTime::from_timestamp(timestamp as i64, 0).ok_or(VcxLedgerError::InvalidInput(
                format!("input status list timestamp is not valid {timestamp}"),
            ))?;

        // assemble query
        let xml_dt = resource_dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let mut query = Url::parse(did)
            .map_err(|e| VcxLedgerError::InvalidInput(format!("cannot parse DID as URL: {e}")))?;
        query
            .query_pairs_mut()
            .append_pair("resourceType", STATUS_LIST_RESOURCE_TYPE)
            .append_pair("resourceName", name)
            .append_pair("resourceVersionTime", &xml_dt);
        let query_url = DidUrl::parse(query.to_string())?;

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

impl AnoncredsLedgerSupport for CheqdAnoncredsLedgerRead {
    fn supports_schema(&self, id: &SchemaId) -> bool {
        let Ok(url) = DidUrl::parse(id.to_string()) else {
            return false;
        };
        url.method() == Some("cheqd")
    }

    fn supports_credential_definition(&self, id: &CredentialDefinitionId) -> bool {
        let Ok(url) = DidUrl::parse(id.to_string()) else {
            return false;
        };
        url.method() == Some("cheqd")
    }

    fn supports_revocation_registry(&self, id: &RevocationRegistryDefinitionId) -> bool {
        let Ok(url) = DidUrl::parse(id.to_string()) else {
            return false;
        };
        url.method() == Some("cheqd")
    }
}

impl Debug for CheqdAnoncredsLedgerRead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CheqdAnoncredsLedgerRead instance")
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    fn default_cheqd_reader() -> CheqdAnoncredsLedgerRead {
        CheqdAnoncredsLedgerRead::new(Arc::new(DidCheqdResolver::new(Default::default())))
    }

    #[test]
    fn test_anoncreds_schema_support() {
        let reader = default_cheqd_reader();

        // qualified cheqd
        assert!(reader.supports_schema(
            &SchemaId::new(
                "did:cheqd:mainnet:7BPMqYgYLQni258J8JPS8K/resources/\
                 6259d357-eeb1-4b98-8bee-12a8390d3497"
            )
            .unwrap()
        ));

        // unqualified
        assert!(!reader.supports_schema(
            &SchemaId::new("7BPMqYgYLQni258J8JPS8K:2:degree schema:46.58.87").unwrap()
        ));
        // qualified sov
        assert!(!reader.supports_schema(
            &SchemaId::new("did:sov:7BPMqYgYLQni258J8JPS8K:2:degree schema:46.58.87").unwrap()
        ));
    }

    #[test]
    fn test_anoncreds_cred_def_support() {
        let reader = default_cheqd_reader();

        // qualified cheqd
        assert!(reader.supports_credential_definition(
            &CredentialDefinitionId::new(
                "did:cheqd:mainnet:7BPMqYgYLQni258J8JPS8K/resources/\
                 6259d357-eeb1-4b98-8bee-12a8390d3497"
            )
            .unwrap()
        ));

        // unqualified
        assert!(!reader.supports_credential_definition(
            &CredentialDefinitionId::new(
                "7BPMqYgYLQni258J8JPS8K:3:CL:70:faber.agent.degree_schema"
            )
            .unwrap()
        ));
        // qualified sov
        assert!(!reader.supports_credential_definition(
            &CredentialDefinitionId::new(
                "did:sov:7BPMqYgYLQni258J8JPS8K:3:CL:70:faber.agent.degree_schema"
            )
            .unwrap()
        ));
    }

    #[test]
    fn test_anoncreds_rev_reg_support() {
        let reader = default_cheqd_reader();

        // qualified cheqd
        assert!(reader.supports_revocation_registry(
            &RevocationRegistryDefinitionId::new(
                "did:cheqd:mainnet:7BPMqYgYLQni258J8JPS8K/resources/\
                 6259d357-eeb1-4b98-8bee-12a8390d3497"
            )
            .unwrap()
        ));

        // unqualified
        assert!(!reader.supports_revocation_registry(
            &RevocationRegistryDefinitionId::new(
                "7BPMqYgYLQni258J8JPS8K:4:7BPMqYgYLQni258J8JPS8K:3:CL:70:faber.agent.\
                 degree_schema:CL_ACCUM:61d5a381-30be-4120-9307-b150b49c203c"
            )
            .unwrap()
        ));
        // qualified sov
        assert!(!reader.supports_revocation_registry(
            &RevocationRegistryDefinitionId::new(
                "did:sov:7BPMqYgYLQni258J8JPS8K:4:7BPMqYgYLQni258J8JPS8K:3:CL:70:faber.agent.\
                 degree_schema:CL_ACCUM:61d5a381-30be-4120-9307-b150b49c203c"
            )
            .unwrap()
        ));
    }
}
