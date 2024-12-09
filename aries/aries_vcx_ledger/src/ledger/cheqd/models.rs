//! Cheqd Ledger data models, derived from https://docs.cheqd.io/product/advanced/anoncreds

use anoncreds_types::data_types::{
    identifiers::{cred_def_id::CredentialDefinitionId, schema_id::SchemaId},
    ledger::{
        cred_def::{CredentialDefinitionData, SignatureType},
        rev_reg_def::{RegistryType, RevocationRegistryDefinitionValue},
        rev_status_list::serde_revocation_list,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheqdAnoncredsSchema {
    pub name: String,
    pub version: String,
    pub attr_names: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheqdAnoncredsCredentialDefinition {
    pub schema_id: SchemaId,
    #[serde(rename = "type")]
    pub signature_type: SignatureType,
    pub tag: String,
    pub value: CredentialDefinitionData,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheqdAnoncredsRevocationRegistryDefinition {
    pub revoc_def_type: RegistryType,
    pub cred_def_id: CredentialDefinitionId,
    pub tag: String,
    pub value: RevocationRegistryDefinitionValue,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheqdAnoncredsRevocationStatusList {
    #[serde(with = "serde_revocation_list")]
    pub revocation_list: bitvec::vec::BitVec,
    #[serde(
        rename = "currentAccumulator",
        alias = "accum",
        skip_serializing_if = "Option::is_none"
    )]
    pub accum: Option<anoncreds_types::cl::Accumulator>,
}
