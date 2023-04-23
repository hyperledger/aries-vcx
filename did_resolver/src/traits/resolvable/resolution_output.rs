use did_doc_builder::schema::did_doc::DIDDocument;
use serde::{Deserialize, Serialize};

use super::resolution_metadata::DIDResolutionMetadata;
use crate::shared_types::did_document_metadata::DIDDocumentMetadata;

// This struct is only returned in the happy case. In the error case, user may convert
// DIDSovError into DIDResolutionMetadata, as DIDResolutionMetadata is be the only
// non-empty field in DIDResolutionOutput in the error case.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DIDResolutionOutput {
    did_document: DIDDocument,
    did_resolution_metadata: DIDResolutionMetadata,
    did_document_metadata: DIDDocumentMetadata,
}

impl DIDResolutionOutput {
    pub fn builder(did_document: DIDDocument) -> DIDResolutionOutputBuilder {
        DIDResolutionOutputBuilder {
            did_document,
            did_resolution_metadata: None,
            did_document_metadata: None,
        }
    }

    pub fn did_document(&self) -> &DIDDocument {
        &self.did_document
    }

    pub fn did_resolution_metadata(&self) -> &DIDResolutionMetadata {
        &self.did_resolution_metadata
    }

    pub fn did_document_metadata(&self) -> &DIDDocumentMetadata {
        &self.did_document_metadata
    }
}

pub struct DIDResolutionOutputBuilder {
    did_document: DIDDocument,
    did_resolution_metadata: Option<DIDResolutionMetadata>,
    did_document_metadata: Option<DIDDocumentMetadata>,
}

impl DIDResolutionOutputBuilder {
    pub fn did_resolution_metadata(
        mut self,
        did_resolution_metadata: DIDResolutionMetadata,
    ) -> Self {
        self.did_resolution_metadata = Some(did_resolution_metadata);
        self
    }

    pub fn did_document_metadata(mut self, did_document_metadata: DIDDocumentMetadata) -> Self {
        self.did_document_metadata = Some(did_document_metadata);
        self
    }

    pub fn build(self) -> DIDResolutionOutput {
        DIDResolutionOutput {
            did_document: self.did_document,
            did_resolution_metadata: self.did_resolution_metadata.unwrap_or_default(),
            did_document_metadata: self.did_document_metadata.unwrap_or_default(),
        }
    }
}
