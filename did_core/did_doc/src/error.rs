use crate::schema::verification_method::{error::KeyDecodingError, VerificationMethodType};

#[derive(Debug)]
pub enum DidDocumentBuilderError {
    CustomError(String),
    InvalidInput(String),
    MissingField(&'static str),
    JsonError(serde_json::Error),
    KeyDecodingError(KeyDecodingError),
    UnsupportedVerificationMethodType(VerificationMethodType),
    PublicKeyError(public_key::PublicKeyError),
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
            DidDocumentBuilderError::UnsupportedVerificationMethodType(vm_type) => {
                write!(f, "Unsupported verification method type: {}", vm_type)
            }
            DidDocumentBuilderError::PublicKeyError(error) => {
                write!(f, "Public key error: {}", error)
            }
            DidDocumentBuilderError::CustomError(string) => {
                write!(f, "Custom DidDocumentBuilderError: {}", string)
            }
            DidDocumentBuilderError::KeyDecodingError(error) => {
                write!(f, "Key decoding error: {}", error)
            }
        }
    }
}

impl std::error::Error for DidDocumentBuilderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DidDocumentBuilderError::JsonError(error) => Some(error),
            DidDocumentBuilderError::PublicKeyError(error) => Some(error),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for DidDocumentBuilderError {
    fn from(error: serde_json::Error) -> Self {
        DidDocumentBuilderError::JsonError(error)
    }
}

impl From<public_key::PublicKeyError> for DidDocumentBuilderError {
    fn from(error: public_key::PublicKeyError) -> Self {
        DidDocumentBuilderError::PublicKeyError(error)
    }
}

impl From<KeyDecodingError> for DidDocumentBuilderError {
    fn from(error: KeyDecodingError) -> Self {
        DidDocumentBuilderError::KeyDecodingError(error)
    }
}
