use std::convert::Infallible;

use did_doc::schema::{
    types::uri::UriWrapperError,
    verification_method::{error::KeyDecodingError, VerificationMethodType},
};

use crate::peer_did::numalgos::kind::NumalgoKind;

#[derive(Debug, thiserror::Error)]
pub enum DidPeerError {
    #[error("DID parser error: {0}")]
    DidParserError(#[from] did_parser_nom::ParseError),
    #[error("Parsing error: {0}")]
    ParsingError(String),
    #[error("DID validation error: {0}")]
    DidValidationError(String),
    #[error("DID document builder error: {0}")]
    DidDocumentBuilderError(#[from] did_doc::error::DidDocumentBuilderError),
    #[error("Invalid key reference: {0}")]
    InvalidKeyReference(String),
    #[error("Invalid service: {0}")]
    InvalidService(String),
    #[error("Unsupported numalgo: {0}")]
    UnsupportedNumalgo(NumalgoKind),
    #[error("Invalid numalgo character: {0}")]
    InvalidNumalgoCharacter(char),
    #[error("Unsupported purpose character: {0}")]
    UnsupportedPurpose(char),
    #[error("Unsupported verification method type: {0}")]
    UnsupportedVerificationMethodType(VerificationMethodType),
    #[error("Base 64 decoding error")]
    Base64DecodingError(#[from] base64::DecodeError),
    #[error("Key decoding error")]
    KeyDecodingError(#[from] KeyDecodingError),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
    #[error("Public key error: {0}")]
    PublicKeyError(#[from] public_key::PublicKeyError),
    #[error("General error: {0}")]
    GeneralError(String),
}

impl From<Infallible> for DidPeerError {
    fn from(_: Infallible) -> Self {
        panic!("Attempted to convert an Infallible error")
    }
}

impl From<UriWrapperError> for DidPeerError {
    fn from(error: UriWrapperError) -> Self {
        DidPeerError::ParsingError(error.to_string())
    }
}
