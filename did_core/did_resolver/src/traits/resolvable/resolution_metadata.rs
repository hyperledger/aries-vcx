use serde::{Deserialize, Serialize};

use super::resolution_error::DidResolutionError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct DidResolutionMetadata {
    content_type: Option<String>,
    error: Option<DidResolutionError>,
}

impl DidResolutionMetadata {
    pub fn builder() -> DidResolutionMetadataBuilder {
        DidResolutionMetadataBuilder::default()
    }

    pub fn content_type(&self) -> Option<&String> {
        self.content_type.as_ref()
    }

    pub fn error(&self) -> Option<&DidResolutionError> {
        self.error.as_ref()
    }
}

#[derive(Default)]
pub struct DidResolutionMetadataBuilder {
    content_type: Option<String>,
    error: Option<DidResolutionError>,
}

impl DidResolutionMetadataBuilder {
    pub fn content_type(mut self, content_type: String) -> Self {
        self.content_type = Some(content_type);
        self
    }

    pub fn error(mut self, error: DidResolutionError) -> Self {
        self.error = Some(error);
        self
    }

    pub fn build(self) -> DidResolutionMetadata {
        DidResolutionMetadata {
            content_type: self.content_type,
            error: self.error,
        }
    }
}
