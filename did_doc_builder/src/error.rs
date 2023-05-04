use url::ParseError;

#[derive(Debug)]
pub enum DidDocumentBuilderError {
    InvalidInput(String),
    MissingField(&'static str),
    JsonError(serde_json::Error),
}

impl std::fmt::Display for DidDocumentBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DidDocumentBuilderError::InvalidInput(input) => {
                write!(f, "Invalid input: {}", input)
            }
            DidDocumentBuilderError::MissingField(field) => {
                write!(f, "Missing field: {}", field)
            }
            DidDocumentBuilderError::JsonError(error) => {
                write!(f, "(De)serialization error: {}", error)
            }
        }
    }
}

impl std::error::Error for DidDocumentBuilderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DidDocumentBuilderError::InvalidInput(_) => None,
            DidDocumentBuilderError::MissingField(_) => None,
            DidDocumentBuilderError::JsonError(error) => Some(error),
        }
    }
}

impl From<serde_json::Error> for DidDocumentBuilderError {
    fn from(error: serde_json::Error) -> Self {
        DidDocumentBuilderError::JsonError(error)
    }
}

impl From<ParseError> for DidDocumentBuilderError {
    fn from(error: ParseError) -> Self {
        DidDocumentBuilderError::InvalidInput(error.to_string())
    }
}
