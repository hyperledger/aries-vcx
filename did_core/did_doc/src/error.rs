use url::ParseError;

use crate::schema::verification_method::VerificationMethodType;

// todo: eliminate CustomError

#[derive(Debug)]
pub enum DidDocumentBuilderError {
    CustomError(String),
    InvalidInput(String),
    MissingField(&'static str),
    UnsupportedPublicKeyField(&'static str),
    JsonError(serde_json::Error),
    PemError(pem::PemError),
    Base58DecodeError(bs58::decode::Error),
    Base64DecodeError(base64::DecodeError),
    HexDecodeError(hex::FromHexError),
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
            DidDocumentBuilderError::UnsupportedPublicKeyField(field) => {
                write!(f, "Unsupported public key field: {}", field)
            }
            DidDocumentBuilderError::JsonError(error) => {
                write!(f, "(De)serialization error: {}", error)
            }
            DidDocumentBuilderError::PemError(error) => {
                write!(f, "PEM error: {}", error)
            }
            DidDocumentBuilderError::Base58DecodeError(error) => {
                write!(f, "Base58 decode error: {}", error)
            }
            DidDocumentBuilderError::Base64DecodeError(error) => {
                write!(f, "Base64 decode error: {}", error)
            }
            DidDocumentBuilderError::HexDecodeError(error) => {
                write!(f, "Hex decode error: {}", error)
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
        }
    }
}

impl std::error::Error for DidDocumentBuilderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DidDocumentBuilderError::JsonError(error) => Some(error),
            DidDocumentBuilderError::PemError(error) => Some(error),
            DidDocumentBuilderError::Base58DecodeError(error) => Some(error),
            DidDocumentBuilderError::Base64DecodeError(error) => Some(error),
            DidDocumentBuilderError::HexDecodeError(error) => Some(error),
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

impl From<pem::PemError> for DidDocumentBuilderError {
    fn from(error: pem::PemError) -> Self {
        DidDocumentBuilderError::PemError(error)
    }
}

impl From<bs58::decode::Error> for DidDocumentBuilderError {
    fn from(error: bs58::decode::Error) -> Self {
        DidDocumentBuilderError::Base58DecodeError(error)
    }
}

impl From<base64::DecodeError> for DidDocumentBuilderError {
    fn from(error: base64::DecodeError) -> Self {
        DidDocumentBuilderError::Base64DecodeError(error)
    }
}

impl From<hex::FromHexError> for DidDocumentBuilderError {
    fn from(error: hex::FromHexError) -> Self {
        DidDocumentBuilderError::HexDecodeError(error)
    }
}

impl From<ParseError> for DidDocumentBuilderError {
    fn from(error: ParseError) -> Self {
        DidDocumentBuilderError::InvalidInput(error.to_string())
    }
}

impl From<public_key::PublicKeyError> for DidDocumentBuilderError {
    fn from(error: public_key::PublicKeyError) -> Self {
        DidDocumentBuilderError::PublicKeyError(error)
    }
}
