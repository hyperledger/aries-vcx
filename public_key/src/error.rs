use thiserror::Error;

#[derive(Debug, Error)]
pub enum PublicKeyError {
    #[error("Base 64 decoding error")]
    Base64DecodingError(#[from] base64::DecodeError),
    #[error("Multibase decoding error")]
    MultibaseDecodingError(#[from] multibase::Error),
    #[error("Varint decoding error")]
    VarintDecodingError(#[from] VarintDecodingError),
    #[error("Unsupported multicodec descriptor: {0}")]
    UnsupportedMulticodecDescriptor(u64),
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

impl From<unsigned_varint::decode::Error> for PublicKeyError {
    fn from(error: unsigned_varint::decode::Error) -> Self {
        Self::VarintDecodingError(error.into())
    }
}
