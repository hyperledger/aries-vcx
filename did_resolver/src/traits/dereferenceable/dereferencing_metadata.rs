use super::dereferencing_error::DidDereferencingError;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DidDereferencingMetadata {
    content_type: Option<String>,
    error: Option<DidDereferencingError>,
}

impl DidDereferencingMetadata {
    pub fn builder() -> DidDereferencingMetadataBuilder {
        DidDereferencingMetadataBuilder::default()
    }

    pub fn content_type(&self) -> Option<&String> {
        self.content_type.as_ref()
    }

    pub fn error(&self) -> Option<&DidDereferencingError> {
        self.error.as_ref()
    }
}

#[derive(Default)]
pub struct DidDereferencingMetadataBuilder {
    content_type: Option<String>,
    error: Option<DidDereferencingError>,
}

impl DidDereferencingMetadataBuilder {
    pub fn content_type(mut self, content_type: String) -> Self {
        self.content_type = Some(content_type);
        self
    }

    pub fn error(mut self, error: DidDereferencingError) -> Self {
        self.error = Some(error);
        self
    }

    pub fn build(self) -> DidDereferencingMetadata {
        DidDereferencingMetadata {
            content_type: self.content_type,
            error: self.error,
        }
    }
}
