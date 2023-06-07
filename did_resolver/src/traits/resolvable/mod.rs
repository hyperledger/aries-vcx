pub mod resolution_error;
pub mod resolution_metadata;
pub mod resolution_options;
pub mod resolution_output;

use crate::error::GenericError;
use async_trait::async_trait;
use did_parser::Did;

use self::{resolution_options::DidResolutionOptions, resolution_output::DidResolutionOutput};

#[async_trait]
pub trait DidResolvable {
    type ExtraFields: Default;

    async fn resolve(
        &self,
        did: &Did,
        options: &DidResolutionOptions,
    ) -> Result<DidResolutionOutput<Self::ExtraFields>, GenericError>;
}
