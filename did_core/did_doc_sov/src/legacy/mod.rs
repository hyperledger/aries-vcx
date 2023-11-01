use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use did_doc::{
    did_parser::{Did, DidUrl},
    schema::{
        did_doc::DidDocument,
        service::Service,
        verification_method::{VerificationMethod, VerificationMethodType},
    },
};
use public_key::{Key, KeyType};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};

use crate::{extra_fields::ExtraFieldsSov, service::legacy::ServiceLegacy};

// TODO: Remove defaults if it turns out they are not needed. Preserved based on the original
// legacy DDO implementation.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
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

// fn resolve_legacy_authentication_key(
//     legacy_authentication: &LegacyAuthentication,
//     legacy_public_keys: &[LegacyKeyAgreement],
// ) -> Result<String, String> {
//     if let Some(fragment) = legacy_authentication.public_key.split('#').last() {
//         Ok(legacy_public_keys
//             .iter()
//             .find(|pk| pk.id.ends_with(fragment))
//             .ok_or_else(|| format!("Public key with id {} not found", fragment))?
//             .public_key_base_58
//             .clone())
//     } else {
//         Ok(legacy_authentication.public_key.clone())
//     }
// }
//
// fn collect_authentication_fingerprints(legacy_ddo: &LegacyDidDoc) -> Result<Vec<String>, String> {
//     let mut authentication_fingerprints = vec![];
//
//     for auth in &legacy_ddo.authentication {
//         let resolved_legacy_authentication_key = match auth.verification_method_type.as_str() {
//             "Ed25519SignatureAuthentication2018" => {
//                 resolve_legacy_authentication_key(auth, &legacy_ddo.public_key)?
//             }
//             "Ed25519Signature2018" => auth.public_key.clone(),
//             _ => {
//                 continue;
//             }
//         };
//
//         let fingerprint = Key::from_base58(&resolved_legacy_authentication_key, KeyType::Ed25519)
//             .map_err(|err| {
//                 format!(
//                     "Error converting legacy authentication key to new key: {:?}, error: {:?}",
//                     auth, err
//                 )
//             })?
//             .fingerprint();
//
//         authentication_fingerprints.push(fingerprint);
//     }
//
//     for vm in &legacy_ddo.public_key {
//         // Ed25519VerificationKey2018 check is used due to aries-vcx using this as key type in the
//         // legacy did doc
//         if !&["Ed25519Signature2018", "Ed25519VerificationKey2018"]
//             .contains(&vm.verification_method_type.as_str())
//         {
//             continue;
//         }
//
//         let fingerprint = Key::from_base58(vm.public_key_base_58.as_str(), KeyType::Ed25519)
//             .map_err(|err| {
//                 format!(
//                     "Error converting legacy public key to new key: {:?}, error: {:?}",
//                     vm, err
//                 )
//             })?
//             .fingerprint();
//
//         if !authentication_fingerprints.contains(&fingerprint) {
//             authentication_fingerprints.push(fingerprint);
//         }
//     }
//
//     Ok(authentication_fingerprints)
// }

// fn collect_encoded_services(legacy_ddo: &LegacyDidDoc) -> Vec<String> {
//     let mut encoded_services = vec![];
//     for service in &legacy_ddo.service {
//         let service = json!({
//             "priority": service.extra().priority(),
//             "r": service.extra().routing_keys(),
//             "recipientKeys": service.extra().recipient_keys(),
//             "s": service.service_endpoint(),
//             "t": service.service_type(),
//         });
//         let service_encoded = STANDARD_NO_PAD.encode(service.to_string().as_bytes());
//         encoded_services.push(service_encoded);
//     }
//     encoded_services
// }

// fn construct_peer_did(
//     authentication_fingerprints: &[String],
//     encoded_services: &[String],
// ) -> Result<Did, String> {
//     // TODO: Perhaps proper ID is did:peer:3 with alsoKnowAs set to did:peer:2 (or vice versa?)
//     let mut did = "did:peer:2".to_string();
//
//     for fingerprint in authentication_fingerprints {
//         did.push_str(&format!(".V{}", fingerprint));
//     }
//
//     for service in encoded_services {
//         did.push_str(&format!(".S{}", service));
//     }
//
//     Did::parse(did).map_err(|err| format!("Error parsing peer did, error: {:?}", err))
// }

// fn construct_new_did_document(
//     legacy_ddo: &LegacyDidDoc,
//     authentication_fingerprints: &[String],
//     did: Did,
// ) -> Result<DidDocument, String> {
//     let mut builder = DidDocument::builder(did.clone());
//
//     for (i, fingerprint) in authentication_fingerprints.iter().enumerate() {
//         let id = DidUrl::from_fragment((i + 1).to_string())
//             .map_err(|err| format!("Error constructing did url from fragment, error: {:?}", err))?;
//         builder = builder.add_verification_method(
//             VerificationMethod::builder(
//                 id,
//                 did.clone(),
//                 VerificationMethodType::Ed25519VerificationKey2018,
//             )
//             .add_public_key_multibase(fingerprint.clone())
//             .build(),
//         );
//     }
//
//     for service in &legacy_ddo.service {
//         builder = builder.add_service(
//             TryInto::<Service>::try_into(service.clone()).map_err(|err| {
//                 format!(
//                     "Error converting legacy service to new service: {:?}, error: {:?}",
//                     service, err
//                 )
//             })?,
//         );
//     }
//
//     Ok(builder.build())
// }

// https://github.com/TimoGlastra/legacy-did-transformation
// fn convert_legacy_ddo_to_new(
//     legacy_ddo: LegacyDidDoc,
// ) -> Result<DidDocument, String> {
//     let authentication_fingerprints = collect_authentication_fingerprints(&legacy_ddo)?;
//     let encoded_services = collect_encoded_services(&legacy_ddo);
//
//     let did = construct_peer_did(&authentication_fingerprints, &encoded_services)?;
//
//     construct_new_did_document(&legacy_ddo, &authentication_fingerprints, did)
// }
//
// pub fn deserialize_legacy_or_new<'de, D>(
//     deserializer: D,
// ) -> Result<DidDocument, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let val = Value::deserialize(deserializer)?;
//     match serde_json::from_value::<LegacyDidDoc>(val.clone()) {
//         Ok(legacy_doc) => {
//             Ok(convert_legacy_ddo_to_new(legacy_doc).map_err(serde::de::Error::custom)?)
//         }
//         Err(_err) => serde_json::from_value::<DidDocument>(val)
//             .map_err(serde::de::Error::custom),
//     }
// }
