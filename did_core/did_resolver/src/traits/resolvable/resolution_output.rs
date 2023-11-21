use did_doc::schema::did_doc::DidDocument;
use serde::{Deserialize, Serialize};

use super::resolution_metadata::DidResolutionMetadata;
use crate::shared_types::did_document_metadata::DidDocumentMetadata;

// This struct is only returned in the happy case. In the error case, user may convert
// DidSovError into DidResolutionMetadata, as DidResolutionMetadata is be the only
// non-empty field in DidResolutionOutput in the error case.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DidResolutionOutput<E: Default> {
    pub did_document: DidDocument<E>,
    pub did_resolution_metadata: DidResolutionMetadata,
    pub did_document_metadata: DidDocumentMetadata,
}

impl<E: Default> DidResolutionOutput<E> {
    pub fn builder(did_document: DidDocument<E>) -> DidResolutionOutputBuilder<E> {
        DidResolutionOutputBuilder {
            did_document,
            did_resolution_metadata: None,
            did_document_metadata: None,
        }
    }

    pub fn did_document(&self) -> &DidDocument<E> {
        &self.did_document
    }

    pub fn did_resolution_metadata(&self) -> &DidResolutionMetadata {
        &self.did_resolution_metadata
    }

    pub fn did_document_metadata(&self) -> &DidDocumentMetadata {
        &self.did_document_metadata
    }
}

pub struct DidResolutionOutputBuilder<E: Default> {
    did_document: DidDocument<E>,
    did_resolution_metadata: Option<DidResolutionMetadata>,
    did_document_metadata: Option<DidDocumentMetadata>,
}

impl<E: Default> DidResolutionOutputBuilder<E> {
    pub fn did_resolution_metadata(
        mut self,
        did_resolution_metadata: DidResolutionMetadata,
    ) -> Self {
        self.did_resolution_metadata = Some(did_resolution_metadata);
        self
    }

    pub fn did_document_metadata(mut self, did_document_metadata: DidDocumentMetadata) -> Self {
        self.did_document_metadata = Some(did_document_metadata);
        self
    }

    pub fn build(self) -> DidResolutionOutput<E> {
        DidResolutionOutput {
            did_document: self.did_document,
            did_resolution_metadata: self.did_resolution_metadata.unwrap_or_default(),
            did_document_metadata: self.did_document_metadata.unwrap_or_default(),
        }
    }
}
