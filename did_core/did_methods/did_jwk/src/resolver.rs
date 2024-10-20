use async_trait::async_trait;
use did_doc::schema::{
    did_doc::DidDocument,
    verification_method::{PublicKeyField, VerificationMethod, VerificationMethodType},
};
use did_parser_nom::{Did, DidUrl};
use did_resolver::{
    error::GenericError,
    traits::resolvable::{resolution_output::DidResolutionOutput, DidResolvable},
};
use serde_json::Value;

use crate::DidJwk;

#[derive(Default)]
pub struct DidJwkResolver;

impl DidJwkResolver {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DidResolvable for DidJwkResolver {
    type DidResolutionOptions = ();

    async fn resolve(
        &self,
        did: &Did,
        _options: &Self::DidResolutionOptions,
    ) -> Result<DidResolutionOutput, GenericError> {
        let did_jwk = DidJwk::try_from(did.to_owned())?;
        let jwk = did_jwk.jwk();

        let jwk_use = jwk.extra.get("use").and_then(Value::as_str);

        let mut did_doc = DidDocument::new(did.to_owned());

        let vm_id = DidUrl::parse(format!("{}#0", did))?;

        let vm = VerificationMethod::builder()
            .id(vm_id.clone())
            .controller(did.clone())
            .verification_method_type(VerificationMethodType::JsonWebKey2020)
            .public_key(PublicKeyField::Jwk {
                public_key_jwk: jwk.clone(),
            })
            .build();
        did_doc.add_verification_method(vm);

        match jwk_use {
            Some("enc") => did_doc.add_key_agreement_ref(vm_id),
            Some("sig") => {
                did_doc.add_assertion_method_ref(vm_id.clone());
                did_doc.add_authentication_ref(vm_id.clone());
                did_doc.add_capability_invocation_ref(vm_id.clone());
                did_doc.add_capability_delegation_ref(vm_id.clone());
            }
            _ => {
                did_doc.add_assertion_method_ref(vm_id.clone());
                did_doc.add_authentication_ref(vm_id.clone());
                did_doc.add_capability_invocation_ref(vm_id.clone());
                did_doc.add_capability_delegation_ref(vm_id.clone());
                did_doc.add_key_agreement_ref(vm_id.clone());
            }
        };

        Ok(DidResolutionOutput::builder(did_doc).build())
    }
}
