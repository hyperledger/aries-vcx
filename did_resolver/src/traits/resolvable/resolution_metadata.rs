use serde::{Deserialize, Serialize};

use super::resolution_error::DIDResolutionError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct DIDResolutionMetadata {
    content_type: Option<String>,
    error: Option<DIDResolutionError>,
}

impl DIDResolutionMetadata {
    pub fn builder() -> DIDResolutionMetadataBuilder {
        DIDResolutionMetadataBuilder::default()
    }

    pub fn content_type(&self) -> Option<&String> {
        self.content_type.as_ref()
    }

    pub fn error(&self) -> Option<&DIDResolutionError> {
        self.error.as_ref()
    }
}

#[derive(Default)]
pub struct DIDResolutionMetadataBuilder {
    content_type: Option<String>,
    error: Option<DIDResolutionError>,
}

impl DIDResolutionMetadataBuilder {
    pub fn content_type(mut self, content_type: String) -> Self {
        self.content_type = Some(content_type);
        self
    }

    pub fn error(mut self, error: DIDResolutionError) -> Self {
        self.error = Some(error);
        self
    }

    pub fn build(self) -> DIDResolutionMetadata {
        DIDResolutionMetadata {
            content_type: self.content_type,
            error: self.error,
        }
    }
}
