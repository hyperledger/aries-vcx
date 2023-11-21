use did_resolver::traits::resolvable::{
    resolution_error::DidResolutionError, resolution_metadata::DidResolutionMetadata,
};

use super::DidSovError;

impl From<&DidSovError> for DidResolutionError {
    fn from(err: &DidSovError) -> Self {
        match err {
            DidSovError::NotFound(_) => DidResolutionError::NotFound,
            DidSovError::MethodNotSupported(_) => DidResolutionError::MethodNotSupported,
            _ => DidResolutionError::InternalError,
        }
    }
}

impl From<&DidSovError> for DidResolutionMetadata {
    fn from(err: &DidSovError) -> Self {
        DidResolutionMetadata::builder().error(err.into()).build()
    }
}
