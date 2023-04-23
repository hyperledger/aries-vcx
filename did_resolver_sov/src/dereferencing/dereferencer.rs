use std::io::Cursor;

use async_trait::async_trait;
use did_resolver::{
    did_parser::ParsedDIDUrl,
    error::GenericError,
    traits::{
        dereferenceable::{
            dereferencing_options::DIDDereferencingOptions,
            dereferencing_output::DIDDereferencingOutput, DIDDereferenceable,
        },
        resolvable::{resolution_options::DIDResolutionOptions, DIDResolvable},
    },
};

use crate::resolution::DIDSovResolver;

use super::utils::dereference_did_document;

#[async_trait]
impl DIDDereferenceable for DIDSovResolver {
    type Output = Cursor<Vec<u8>>;

    async fn dereference(
        &self,
        did_url: &ParsedDIDUrl,
        _options: &DIDDereferencingOptions,
    ) -> Result<DIDDereferencingOutput<Self::Output>, GenericError> {
        let resolution_output = self
            .resolve(&did_url.try_into()?, &DIDResolutionOptions::default())
            .await?;

        dereference_did_document(&resolution_output, &did_url).map_err(|err| err.into())
    }
}
