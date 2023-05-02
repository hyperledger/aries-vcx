pub mod dereferencing_error;
pub mod dereferencing_metadata;
pub mod dereferencing_options;
pub mod dereferencing_output;

use std::io::Read;

use crate::{error::GenericError, traits::resolvable::DIDResolvable};
use async_trait::async_trait;
use did_parser::ParsedDIDUrl;

use self::{
    dereferencing_options::DIDDereferencingOptions, dereferencing_output::DIDDereferencingOutput,
};

#[async_trait]
pub trait DIDDereferenceable: DIDResolvable {
    type Output: Read + Send + Sync;

    async fn dereference(
        &self,
        did: &ParsedDIDUrl,
        options: &DIDDereferencingOptions,
    ) -> Result<DIDDereferencingOutput<Self::Output>, GenericError>;
}
