use async_trait::async_trait;
use base64::{
    alphabet,
    engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig},
    Engine,
};
use did_doc::schema::{
    did_doc::DidDocument,
    types::jsonwebkey::JsonWebKey,
    verification_method::{PublicKeyField, VerificationMethod, VerificationMethodType},
};
use did_parser_nom::{Did, DidUrl};
use did_resolver::{
    error::GenericError,
    traits::resolvable::{resolution_output::DidResolutionOutput, DidResolvable},
};
use serde_json::Value;

/// A default [GeneralPurposeConfig] configuration with a [decode_padding_mode] of
/// [DecodePaddingMode::Indifferent]
const LENIENT_PAD: GeneralPurposeConfig = GeneralPurposeConfig::new()
    .with_encode_padding(false)
    .with_decode_padding_mode(DecodePaddingMode::Indifferent);

/// A [GeneralPurpose] engine using the [alphabet::URL_SAFE] base64 alphabet and
/// [DecodePaddingMode::Indifferent] config to decode both padded and unpadded.
const URL_SAFE_LENIENT: GeneralPurpose = GeneralPurpose::new(&alphabet::URL_SAFE, LENIENT_PAD);

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
        let did = did.to_owned();
        // TODO - all unwraps
        let Some("jwk") = did.method() else {
            todo!();
        };
        let jwk_base64 = did.id();
        let jwk_bytes = URL_SAFE_LENIENT.decode(jwk_base64)?;

        let jwk: JsonWebKey = serde_json::from_slice(&jwk_bytes)?;

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
