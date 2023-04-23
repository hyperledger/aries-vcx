use did_resolver::traits::resolvable::{
    resolution_error::DIDResolutionError, resolution_metadata::DIDResolutionMetadata,
};

use super::DIDSovError;

impl From<&DIDSovError> for DIDResolutionError {
    fn from(err: &DIDSovError) -> Self {
        match err {
            DIDSovError::NotFound(_) => DIDResolutionError::NotFound,
            DIDSovError::MethodNotSupported(_) => DIDResolutionError::MethodNotSupported,
            _ => DIDResolutionError::InternalError,
        }
    }
}

impl From<&DIDSovError> for DIDResolutionMetadata {
    fn from(err: &DIDSovError) -> Self {
        DIDResolutionMetadata::builder().error(err.into()).build()
    }
}
