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
// TODO: Enable once associated type defaults are stable: https://github.com/rust-lang/rust/issues/29661
// #[cfg_attr(test, mockall::automock)]
pub trait DIDDereferenceable: DIDResolvable {
    type Output: Read + Send + Sync;

    async fn dereference(
        &self,
        did: &ParsedDIDUrl,
        options: &DIDDereferencingOptions,
    ) -> Result<DIDDereferencingOutput<Self::Output>, GenericError>;
}

#[async_trait]
// #[cfg_attr(test, mockall::automock)]
pub trait DIDDereferenceableMut: DIDDereferenceable {
    type Output: Read + Send + Sync;

    async fn dereference_mut(
        &mut self,
        did: &ParsedDIDUrl,
        options: &DIDDereferencingOptions,
    ) -> Result<DIDDereferencingOutput<<Self as DIDDereferenceableMut>::Output>, GenericError>;
}
