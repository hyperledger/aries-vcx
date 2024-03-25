use did_doc::schema::did_doc::DidDocument;
use serde::{Deserialize, Serialize};

use super::resolution_metadata::DidResolutionMetadata;
use crate::shared_types::did_document_metadata::DidDocumentMetadata;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DidResolutionOutput {
    pub did_document: DidDocument,
    pub did_resolution_metadata: DidResolutionMetadata,
    pub did_document_metadata: DidDocumentMetadata,
}

impl DidResolutionOutput {
    pub fn builder(did_document: DidDocument) -> DidResolutionOutputBuilder {
        DidResolutionOutputBuilder {
            did_document,
            did_resolution_metadata: None,
            did_document_metadata: None,
        }
    }
}

pub struct DidResolutionOutputBuilder {
    did_document: DidDocument,
    did_resolution_metadata: Option<DidResolutionMetadata>,
    did_document_metadata: Option<DidDocumentMetadata>,
}

impl DidResolutionOutputBuilder {
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

    pub fn build(self) -> DidResolutionOutput {
        DidResolutionOutput {
            did_document: self.did_document,
            did_resolution_metadata: self.did_resolution_metadata.unwrap_or_default(),
            did_document_metadata: self.did_document_metadata.unwrap_or_default(),
        }
    }
}
