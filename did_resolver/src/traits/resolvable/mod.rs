pub mod resolution_error;
pub mod resolution_metadata;
pub mod resolution_options;
pub mod resolution_output;

use crate::error::GenericError;
use async_trait::async_trait;
use did_parser::ParsedDID;

use self::{resolution_options::DIDResolutionOptions, resolution_output::DIDResolutionOutput};

#[async_trait]
pub trait DIDResolvable {
    async fn resolve(
        &self,
        did: &ParsedDID,
        options: &DIDResolutionOptions,
    ) -> Result<DIDResolutionOutput, GenericError>;
}
