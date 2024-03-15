pub mod resolution_error;
pub mod resolution_metadata;
pub mod resolution_output;

use async_trait::async_trait;
use did_parser_nom::Did;

use self::resolution_output::DidResolutionOutput;
use crate::error::GenericError;

#[async_trait]
pub trait DidResolvable {
    type DidResolutionOptions: Default;

    async fn resolve(
        &self,
        did: &Did,
        options: &Self::DidResolutionOptions,
    ) -> Result<DidResolutionOutput, GenericError>;
}
