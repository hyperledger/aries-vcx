use std::error::Error;

#[derive(Debug)]
pub enum DIDResolverRegistryError {
    UnsupportedMethod,
}

impl std::fmt::Display for DIDResolverRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DIDResolverRegistryError::UnsupportedMethod => write!(f, "Unsupported DID method"),
        }
    }
}

impl Error for DIDResolverRegistryError {}
