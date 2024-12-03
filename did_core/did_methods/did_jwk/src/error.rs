use thiserror::Error;

#[derive(Debug, Error)]
pub enum DidJwkError {
    #[error("DID method not supported: {0}")]
    MethodNotSupported(String),
    #[error("Base64 encoding error: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("Serde JSON error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Public key error: {0}")]
    PublicKeyError(#[from] public_key::PublicKeyError),
    #[error("DID parser error: {0}")]
    DidParserError(#[from] did_parser_nom::ParseError),
}
