use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use public_key::{Key, KeyType};
use serde::{Deserialize, Deserializer, Serialize};

use did_doc::{
    did_parser::{Did, DidUrl},
    schema::{
        did_doc::DidDocument,
        service::Service,
        verification_method::{VerificationMethod, VerificationMethodType},
    },
};
use serde_json::{json, Value};

use crate::{extra_fields::ExtraFieldsSov, service::legacy::ServiceLegacy};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct LegacyDidDoc {
    id: Did,
    #[serde(default)]
    #[serde(rename = "publicKey")]
    public_key: Vec<LegacyKeyAgreement>,
    #[serde(default)]
    authentication: Vec<LegacyAuthentication>,
    service: Vec<ServiceLegacy>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct LegacyKeyAgreement {
    id: String,
    #[serde(rename = "type")]
    verification_method_type: String,
    controller: String,
    #[serde(rename = "publicKeyBase58")]
    public_key_base_58: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct LegacyAuthentication {
    #[serde(rename = "type")]
    verification_method_type: String,
    #[serde(rename = "publicKey")]
    public_key: String,
}

fn resolve_legacy_authentication_key(
    legacy_authentication: &LegacyAuthentication,
    legacy_public_keys: &[LegacyKeyAgreement],
) -> Result<String, String> {
    if let Some(fragment) = legacy_authentication.public_key.split('#').last() {
        Ok(legacy_public_keys
            .iter()
            .find(|pk| pk.id.ends_with(fragment))
            .ok_or_else(|| format!("Public key with id {} not found", fragment))?
            .public_key_base_58
            .clone())
    } else {
        Ok(legacy_authentication.public_key.clone())
    }
}

fn collect_authentication_fingerprints(legacy_ddo: &LegacyDidDoc) -> Result<Vec<String>, String> {
    let mut authentication_fingerprints = vec![];

    for auth in &legacy_ddo.authentication {
        let resolved_legacy_authentication_key = match auth.verification_method_type.as_str() {
            "Ed25519SignatureAuthentication2018" => resolve_legacy_authentication_key(auth, &legacy_ddo.public_key)?,
            "Ed25519Signature2018" => auth.public_key.clone(),
            _ => {
                continue;
            }
        };

        let fingerprint = Key::from_base58(&resolved_legacy_authentication_key, KeyType::Ed25519)
            .map_err(|err| {
                format!(
                    "Error converting legacy authentication key to new key: {:?}, error: {:?}",
                    auth, err
                )
            })?
            .fingerprint();
        authentication_fingerprints.push(fingerprint);
    }

    for vm in &legacy_ddo.public_key {
        if vm.verification_method_type != "Ed25519Signature2018" {
            continue;
        }

        let fingerprint = Key::from_base58(vm.public_key_base_58.as_str(), KeyType::Ed25519)
            .map_err(|err| {
                format!(
                    "Error converting legacy public key to new key: {:?}, error: {:?}",
                    vm, err
                )
            })?
            .fingerprint();
        if !authentication_fingerprints.contains(&fingerprint) {
            authentication_fingerprints.push(fingerprint);
        }
    }

    Ok(authentication_fingerprints)
}

fn collect_encoded_services(legacy_ddo: &LegacyDidDoc) -> Vec<String> {
    let mut encoded_services = vec![];
    for service in &legacy_ddo.service {
        let service = json!({
            "priority": service.extra().priority(),
            "r": service.extra().routing_keys(),
            "recipientKeys": service.extra().recipient_keys(),
            "s": service.service_endpoint(),
            "t": service.service_type(),
        });
        let service_encoded = STANDARD_NO_PAD.encode(service.to_string().as_bytes());
        encoded_services.push(service_encoded);
    }
    encoded_services
}

fn construct_peer_did(authentication_fingerprints: &[String], encoded_services: &[String]) -> Result<Did, String> {
    let mut did = "did:peer:2".to_string();

    for fingerprint in authentication_fingerprints {
        did.push_str(&format!(".V{}", fingerprint));
    }

    for service in encoded_services {
        did.push_str(&format!(".S{}", service));
    }

    Did::parse(did).map_err(|err| format!("Error parsing peer did, error: {:?}", err))
}

// https://github.com/TimoGlastra/legacy-did-transformation
fn convert_legacy_ddo_to_new(legacy_ddo: LegacyDidDoc) -> Result<DidDocument<ExtraFieldsSov>, String> {
    let authentication_fingerprints = collect_authentication_fingerprints(&legacy_ddo)?;
    let encoded_services = collect_encoded_services(&legacy_ddo);

    let did = construct_peer_did(&authentication_fingerprints, &encoded_services)?;

    let mut builder = DidDocument::builder(did.clone());

    for (i, fingerprint) in authentication_fingerprints.iter().enumerate() {
        let id = DidUrl::from_fragment(i.to_string()).unwrap();
        builder = builder.add_verification_method(
            VerificationMethod::builder(
                id,
                did.clone().into(),
                VerificationMethodType::Ed25519VerificationKey2018,
            )
            .add_public_key_multibase(fingerprint.clone())
            .build(),
        );
    }

    for service in &legacy_ddo.service {
        builder = builder.add_service(TryInto::<Service<ExtraFieldsSov>>::try_into(service.clone()).map_err(
            |err| {
                format!(
                    "Error converting legacy service to new service: {:?}, error: {:?}",
                    service, err
                )
            },
        )?);
    }

    Ok(builder.build())
}

pub fn deserialize_legacy_or_new<'de, D>(deserializer: D) -> Result<DidDocument<ExtraFieldsSov>, D::Error>
where
    D: Deserializer<'de>,
{
    let val = Value::deserialize(deserializer)?;

    match serde_json::from_value::<LegacyDidDoc>(val.clone()) {
        Ok(legacy_doc) => Ok(convert_legacy_ddo_to_new(legacy_doc).map_err(serde::de::Error::custom)?),
        Err(_err) => {
            println!("Error deserializing legacy did doc: {:?}", _err);
            serde_json::from_value::<DidDocument<ExtraFieldsSov>>(val).map_err(serde::de::Error::custom)
        }
    }
}
