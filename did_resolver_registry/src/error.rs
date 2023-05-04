use std::error::Error;

#[derive(Debug)]
pub enum DidResolverRegistryError {
    UnsupportedMethod,
}

impl std::fmt::Display for DidResolverRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DidResolverRegistryError::UnsupportedMethod => write!(f, "Unsupported DID method"),
        }
    }
}

impl Error for DidResolverRegistryError {}
