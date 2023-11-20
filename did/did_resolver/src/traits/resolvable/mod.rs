pub mod resolution_error;
pub mod resolution_metadata;
pub mod resolution_options;
pub mod resolution_output;

use async_trait::async_trait;
use did_parser::Did;

use self::{resolution_options::DidResolutionOptions, resolution_output::DidResolutionOutput};
use crate::error::GenericError;

#[async_trait]
pub trait DidResolvable {
    type ExtraFieldsService: Default;
    type ExtraFieldsOptions: Default;

    async fn resolve(
        &self,
        did: &Did,
        options: &DidResolutionOptions<Self::ExtraFieldsOptions>,
    ) -> Result<DidResolutionOutput<Self::ExtraFieldsService>, GenericError>;
}
