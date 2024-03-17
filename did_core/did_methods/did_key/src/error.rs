use thiserror::Error;

#[derive(Debug, Error)]
pub enum DidKeyError {
    #[error("Public key error: {0}")]
    PublicKeyError(#[from] public_key::PublicKeyError),
    #[error("DID parser error: {0}")]
    DidParserError(#[from] did_parser_nom::ParseError),
}
