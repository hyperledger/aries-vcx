use std::convert::Infallible;

use did_doc::schema::verification_method::VerificationMethodType;
use thiserror::Error;

use crate::peer_did::numalgos::NumalgoKind;

#[derive(Debug, Error)]
pub enum DidPeerError {
    #[error("DID parser error: {0}")]
    DidParserError(#[from] did_parser::ParseError),
    #[error("DID validation error: {0}")]
    DidValidationError(String),
    #[error("DID document builder error: {0}")]
    DidDocumentBuilderError(#[from] did_doc::error::DidDocumentBuilderError),
    #[error("Invalid key reference: {0}")]
    InvalidKeyReference(String),
    #[error("Invalid service type")]
    InvalidServiceType,
    #[error("Sovrin DID document builder error: {0}")]
    DidDocumentSovBuilderError(#[from] did_doc_sov::error::DidDocumentSovError),
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
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
    #[error("Public key error: {0}")]
    PublicKeyError(#[from] public_key::PublicKeyError),
}

impl From<Infallible> for DidPeerError {
    fn from(_: Infallible) -> Self {
        panic!("Attempted to convert an Infallible error")
    }
}
