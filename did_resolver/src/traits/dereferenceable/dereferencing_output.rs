use crate::shared_types::did_document_metadata::DidDocumentMetadata;
use std::io::Read;

use super::dereferencing_metadata::DidDereferencingMetadata;

pub struct DidDereferencingOutput<R: Read + Send + Sync> {
    dereferencing_metadata: DidDereferencingMetadata,
    content_stream: R,
    content_metadata: DidDocumentMetadata,
}

impl<R> DidDereferencingOutput<R>
where
    R: Read + Send + Sync,
{
    pub fn builder(content_stream: R) -> DidDDereferencingOutputBuilder<R> {
        DidDDereferencingOutputBuilder {
            dereferencing_metadata: None,
            content_stream,
            content_metadata: None,
        }
    }

    pub fn dereferencing_metadata(&self) -> &DidDereferencingMetadata {
        &self.dereferencing_metadata
    }

    pub fn content_stream(&self) -> &R {
        &self.content_stream
    }

    pub fn content_metadata(&self) -> &DidDocumentMetadata {
        &self.content_metadata
    }
}

pub struct DidDDereferencingOutputBuilder<R: Read + Send + Sync> {
    dereferencing_metadata: Option<DidDereferencingMetadata>,
    content_stream: R,
    content_metadata: Option<DidDocumentMetadata>,
}

impl<R> DidDDereferencingOutputBuilder<R>
where
    R: Read + Send + Sync,
{
    pub fn dereferencing_metadata(
        mut self,
        dereferencing_metadata: DidDereferencingMetadata,
    ) -> Self {
        self.dereferencing_metadata = Some(dereferencing_metadata);
        self
    }

    pub fn content_metadata(mut self, content_metadata: DidDocumentMetadata) -> Self {
        self.content_metadata = Some(content_metadata);
        self
    }

    pub fn build(self) -> DidDereferencingOutput<R> {
        DidDereferencingOutput {
            dereferencing_metadata: self.dereferencing_metadata.unwrap_or_default(),
            content_stream: self.content_stream,
            content_metadata: self.content_metadata.unwrap_or_default(),
        }
    }
}
