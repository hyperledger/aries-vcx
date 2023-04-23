use crate::shared_types::did_document_metadata::DIDDocumentMetadata;
use std::io::Read;

use super::dereferencing_metadata::DIDDereferencingMetadata;

pub struct DIDDereferencingOutput<R: Read + Send + Sync> {
    dereferencing_metadata: DIDDereferencingMetadata,
    content_stream: R,
    content_metadata: DIDDocumentMetadata,
}

impl<R> DIDDereferencingOutput<R>
where
    R: Read + Send + Sync,
{
    pub fn builder(content_stream: R) -> DIDDereferencingOutputBuilder<R> {
        DIDDereferencingOutputBuilder {
            dereferencing_metadata: None,
            content_stream,
            content_metadata: None,
        }
    }

    pub fn dereferencing_metadata(&self) -> &DIDDereferencingMetadata {
        &self.dereferencing_metadata
    }

    pub fn content_stream(&self) -> &R {
        &self.content_stream
    }

    pub fn content_metadata(&self) -> &DIDDocumentMetadata {
        &self.content_metadata
    }
}

pub struct DIDDereferencingOutputBuilder<R: Read + Send + Sync> {
    dereferencing_metadata: Option<DIDDereferencingMetadata>,
    content_stream: R,
    content_metadata: Option<DIDDocumentMetadata>,
}

impl<R> DIDDereferencingOutputBuilder<R>
where
    R: Read + Send + Sync,
{
    pub fn dereferencing_metadata(
        mut self,
        dereferencing_metadata: DIDDereferencingMetadata,
    ) -> Self {
        self.dereferencing_metadata = Some(dereferencing_metadata);
        self
    }

    pub fn content_metadata(mut self, content_metadata: DIDDocumentMetadata) -> Self {
        self.content_metadata = Some(content_metadata);
        self
    }

    pub fn build(self) -> DIDDereferencingOutput<R> {
        DIDDereferencingOutput {
            dereferencing_metadata: self.dereferencing_metadata.unwrap_or_default(),
            content_stream: self.content_stream,
            content_metadata: self.content_metadata.unwrap_or_default(),
        }
    }
}
