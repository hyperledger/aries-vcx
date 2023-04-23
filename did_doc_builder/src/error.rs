#[derive(Debug)]
pub enum DIDDocumentBuilderError {
    InvalidInput(String),
    MissingField(&'static str),
    SerdeError(serde_json::Error),
}

impl std::fmt::Display for DIDDocumentBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DIDDocumentBuilderError::InvalidInput(input) => {
                write!(f, "Invalid input: {}", input)
            }
            DIDDocumentBuilderError::MissingField(field) => {
                write!(f, "Missing field: {}", field)
            }
            DIDDocumentBuilderError::SerdeError(error) => {
                write!(f, "(De)serialization error: {}", error)
            }
        }
    }
}

impl std::error::Error for DIDDocumentBuilderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DIDDocumentBuilderError::InvalidInput(_) => None,
            DIDDocumentBuilderError::MissingField(_) => None,
            DIDDocumentBuilderError::SerdeError(error) => Some(error),
        }
    }
}

impl From<serde_json::Error> for DIDDocumentBuilderError {
    fn from(error: serde_json::Error) -> Self {
        DIDDocumentBuilderError::SerdeError(error)
    }
}
