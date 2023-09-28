pub mod dereferencing_error;
pub mod dereferencing_metadata;
pub mod dereferencing_options;
pub mod dereferencing_output;

use std::io::Read;

use async_trait::async_trait;
use did_parser::DidUrl;

use self::{
    dereferencing_options::DidDereferencingOptions, dereferencing_output::DidDereferencingOutput,
};
use crate::{error::GenericError, traits::resolvable::DidResolvable};

#[async_trait]
pub trait DidDereferenceable: DidResolvable {
    type Output: Read + Send + Sync;

    async fn dereference(
        &self,
        did: &DidUrl,
        options: &DidDereferencingOptions,
    ) -> Result<DidDereferencingOutput<Self::Output>, GenericError>;
}
