use std::collections::HashSet;

use chrono::{DateTime, Utc};
use did_parser::ParsedDID;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct DIDDocumentMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    created: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    updated: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deactivated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_update: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_version_id: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    equivalent_id: Vec<ParsedDID>,
    #[serde(skip_serializing_if = "Option::is_none")]
    canonical_id: Option<ParsedDID>,
}

impl DIDDocumentMetadata {
    pub fn builder() -> DIDDocumentMetadataBuilder {
        DIDDocumentMetadataBuilder::default()
    }

    pub fn created(&self) -> Option<DateTime<Utc>> {
        self.created
    }

    pub fn updated(&self) -> Option<DateTime<Utc>> {
        self.updated
    }

    pub fn deactivated(&self) -> Option<bool> {
        self.deactivated
    }

    pub fn next_update(&self) -> Option<DateTime<Utc>> {
        self.next_update
    }

    pub fn version_id(&self) -> Option<&String> {
        self.version_id.as_ref()
    }

    pub fn next_version_id(&self) -> Option<&String> {
        self.next_version_id.as_ref()
    }

    pub fn equivalent_id(&self) -> &[ParsedDID] {
        self.equivalent_id.as_ref()
    }

    pub fn canonical_id(&self) -> Option<&ParsedDID> {
        self.canonical_id.as_ref()
    }
}

#[derive(Default)]
pub struct DIDDocumentMetadataBuilder {
    created: Option<DateTime<Utc>>,
    updated: Option<DateTime<Utc>>,
    deactivated: Option<bool>,
    next_update: Option<DateTime<Utc>>,
    version_id: Option<String>,
    next_version_id: Option<String>,
    equivalent_id: HashSet<ParsedDID>,
    canonical_id: Option<ParsedDID>,
}

impl DIDDocumentMetadataBuilder {
    pub fn created(mut self, created: DateTime<Utc>) -> Self {
        self.created = Some(created);
        self
    }

    pub fn updated(mut self, updated: DateTime<Utc>) -> Self {
        self.updated = Some(updated);
        self
    }

    pub fn deactivated(mut self, deactivated: bool) -> Self {
        self.deactivated = Some(deactivated);
        self
    }

    pub fn next_update(mut self, next_update: DateTime<Utc>) -> Self {
        self.next_update = Some(next_update);
        self
    }

    pub fn version_id(mut self, version_id: String) -> Self {
        self.version_id = Some(version_id);
        self
    }

    pub fn next_version_id(mut self, next_version_id: String) -> Self {
        self.next_version_id = Some(next_version_id);
        self
    }

    pub fn add_equivalent_id(mut self, equivalent_id: ParsedDID) -> Self {
        self.equivalent_id.insert(equivalent_id);
        self
    }

    pub fn canonical_id(mut self, canonical_id: ParsedDID) -> Self {
        self.canonical_id = Some(canonical_id);
        self
    }

    pub fn build(self) -> DIDDocumentMetadata {
        DIDDocumentMetadata {
            created: self.created,
            updated: self.updated,
            deactivated: self.deactivated,
            next_update: self.next_update,
            version_id: self.version_id,
            next_version_id: self.next_version_id,
            equivalent_id: self.equivalent_id.into_iter().collect(),
            canonical_id: self.canonical_id,
        }
    }
}
