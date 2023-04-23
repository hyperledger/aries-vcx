use super::dereferencing_error::DIDDereferencingError;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DIDDereferencingMetadata {
    content_type: Option<String>,
    error: Option<DIDDereferencingError>,
}

impl DIDDereferencingMetadata {
    pub fn builder() -> DIDDereferencingMetadataBuilder {
        DIDDereferencingMetadataBuilder::default()
    }

    pub fn content_type(&self) -> Option<&String> {
        self.content_type.as_ref()
    }

    pub fn error(&self) -> Option<&DIDDereferencingError> {
        self.error.as_ref()
    }
}

#[derive(Default)]
pub struct DIDDereferencingMetadataBuilder {
    content_type: Option<String>,
    error: Option<DIDDereferencingError>,
}

impl DIDDereferencingMetadataBuilder {
    pub fn content_type(mut self, content_type: String) -> Self {
        self.content_type = Some(content_type);
        self
    }

    pub fn error(mut self, error: DIDDereferencingError) -> Self {
        self.error = Some(error);
        self
    }

    pub fn build(self) -> DIDDereferencingMetadata {
        DIDDereferencingMetadata {
            content_type: self.content_type,
            error: self.error,
        }
    }
}
