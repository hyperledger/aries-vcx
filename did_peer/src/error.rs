use std::convert::Infallible;

use did_doc::schema::verification_method::VerificationMethodType;
use thiserror::Error;

use crate::peer_did::Numalgo;

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
    UnsupportedNumalgo(Numalgo),
    #[error("Invalid numalgo character: {0}")]
    InvalidNumalgoCharacter(char),
    #[error("Unsupported purpose character: {0}")]
    UnsupportedPurpose(char),
    #[error("Unsupported multicodec descriptor: {0}")]
    UnsupportedMulticodecDescriptor(u64),
    #[error("Unsupported verification method type: {0}")]
    UnsupportedVerificationMethodType(VerificationMethodType),
    #[error("Base 64 decoding error")]
    Base64DecodingError(#[from] base64::DecodeError),
    #[error("Multibase decoding error")]
    MultibaseDecodingError(#[from] multibase::Error),
    #[error("Varint decoding error")]
    VarintDecodingError(#[from] VarintDecodingError),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
}

impl From<Infallible> for DidPeerError {
    fn from(_: Infallible) -> Self {
        panic!("Attempted to convert an Infallible error")
    }
}

#[derive(Debug, Error)]
pub struct VarintDecodingError(unsigned_varint::decode::Error);

impl std::fmt::Display for VarintDecodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Varint decoding error: {}", self.0)
    }
}

impl From<unsigned_varint::decode::Error> for VarintDecodingError {
    fn from(error: unsigned_varint::decode::Error) -> Self {
        Self(error)
    }
}

impl From<unsigned_varint::decode::Error> for DidPeerError {
    fn from(error: unsigned_varint::decode::Error) -> Self {
        Self::VarintDecodingError(error.into())
    }
}
