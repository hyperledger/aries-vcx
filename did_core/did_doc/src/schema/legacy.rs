use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use did_parser::{Did, DidUrl};
use public_key::{Key, KeyType};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::schema::{
    did_doc::DidDocument,
    service::Service,
    verification_method::{VerificationMethod, VerificationMethodType},
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, display_as_json::Display)]
#[serde(deny_unknown_fields)]
pub struct LegacyDidDoc {
    id: Did,
    #[serde(default)]
    #[serde(rename = "publicKey")]
    public_key: Vec<LegacyKeyAgreement>,
    #[serde(default)]
    authentication: Vec<LegacyAuthentication>,
    service: Vec<Service>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, display_as_json::Display)]
pub struct LegacyKeyAgreement {
    id: String,
    #[serde(rename = "type")]
    verification_method_type: String,
    controller: String,
    #[serde(rename = "publicKeyBase58")]
    public_key_base_58: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, display_as_json::Display)]
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
            "Ed25519SignatureAuthentication2018" => {
                resolve_legacy_authentication_key(auth, &legacy_ddo.public_key)?
            }
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
        // Ed25519VerificationKey2018 check is used due to aries-vcx using this as key type in
        // the legacy did doc
        if !&["Ed25519Signature2018", "Ed25519VerificationKey2018"]
            .contains(&vm.verification_method_type.as_str())
        {
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
        let priority = service.extra_field_priority().unwrap_or(0);
        let routing_keys = service.extra_field_routing_keys().unwrap_or(vec![]);
        let recipient_keys = service.extra_field_recipient_keys().unwrap_or(vec![]);
        let service_endpoint = service.service_endpoint().to_string();
        let service_type = service.service_types().first().unwrap().to_string();
        let service = json!({
            "priority": priority,
            "r": routing_keys,
            "recipientKeys": recipient_keys,
            "s": service_endpoint,
            "t": service_type,
        });
        let service_encoded = STANDARD_NO_PAD.encode(service.to_string().as_bytes());
        encoded_services.push(service_encoded);
    }
    encoded_services
}

fn construct_peer_did(
    authentication_fingerprints: &[String],
    encoded_services: &[String],
) -> Result<Did, String> {
    // TODO: Perhaps proper ID is did:peer:3 with alsoKnowAs set to did:peer:2 (or vice versa?)
    let mut did = "did:peer:2".to_string();

    for fingerprint in authentication_fingerprints {
        did.push_str(&format!(".V{}", fingerprint));
    }

    for service in encoded_services {
        did.push_str(&format!(".S{}", service));
    }

    Did::parse(did).map_err(|err| format!("Error parsing peer did, error: {:?}", err))
}

fn construct_new_did_document(
    legacy_ddo: &LegacyDidDoc,
    authentication_fingerprints: &[String],
    did: Did,
) -> Result<DidDocument, String> {
    let mut builder = DidDocument::builder(did.clone());

    for (i, fingerprint) in authentication_fingerprints.iter().enumerate() {
        let id = DidUrl::from_fragment((i + 1).to_string())
            .map_err(|err| format!("Error constructing did url from fragment, error: {:?}", err))?;
        builder = builder.add_verification_method(
            VerificationMethod::builder(
                id,
                did.clone(),
                VerificationMethodType::Ed25519VerificationKey2018,
            )
            .add_public_key_multibase(fingerprint.clone())
            .build(),
        );
    }

    for service in &legacy_ddo.service {
        builder = builder.add_service(service.clone());
    }

    Ok(builder.build())
}

// https://github.com/TimoGlastra/legacy-did-transformation
fn convert_legacy_ddo_to_new(legacy_ddo: LegacyDidDoc) -> Result<DidDocument, String> {
    let authentication_fingerprints = collect_authentication_fingerprints(&legacy_ddo)?;
    let encoded_services = collect_encoded_services(&legacy_ddo);
    let did = construct_peer_did(&authentication_fingerprints, &encoded_services)?;
    construct_new_did_document(&legacy_ddo, &authentication_fingerprints, did)
}

pub fn deserialize_legacy_or_new_diddoc_value(val: Value) -> Result<DidDocument, String> {
    match serde_json::from_value::<LegacyDidDoc>(val.clone()) {
        Ok(legacy_doc) => convert_legacy_ddo_to_new(legacy_doc),
        Err(_err) => Ok(serde_json::from_value::<DidDocument>(val).map_err(|err| {
            format!(
                "Error deserializing did document from value, error: {:?}",
                err
            )
        })?),
    }
}

pub fn deserialize_legacy_or_new_diddoc_str(val: String) -> Result<DidDocument, String> {
    let value = serde_json::from_str::<Value>(&val)
        .map_err(|err| format!("Error deserializing did doc value, error: {:?}", err))?;
    deserialize_legacy_or_new_diddoc_value(value)
}

#[cfg(test)]
mod tests {
    use crate::schema::{
        did_doc::DidDocument, legacy::deserialize_legacy_or_new_diddoc_str,
        service::service_key_kind::ServiceKeyKind,
    };

    const LEGACY_DID_DOC_JSON: &str = r#"
    {
       "id": "2ZHFFhzA2XtTD6hJqzL7ux",
       "publicKey": [
           {
               "id": "1",
               "type": "Ed25519VerificationKey2018",
               "controller": "2ZHFFhzA2XtTD6hJqzL7ux",
               "publicKeyBase58": "rCw3x5h1jS6gPo7rRrt3EYbXXe5nNjnGbdf1jAwUxuj"
           }
       ],
       "authentication": [
           {
               "type": "Ed25519SignatureAuthentication2018",
               "publicKey": "2ZHFFhzA2XtTD6hJqzL7ux#1"
           }
       ],
       "service": [
           {
               "id": "did:example:123456789abcdefghi;indy",
               "type": "IndyAgent",
               "priority": 0,
               "recipientKeys": [
                   "2ZHFFhzA2XtTD6hJqzL7ux#1"
               ],
               "routingKeys": [
                   "8Ps2WosJ9AV1eXPoJKsEJdM3NchPhSyS8qFt6LQUTKv2",
                   "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR"
               ],
               "serviceEndpoint": "http://localhost:8080/agency/msg"
           }
       ]
    }"#;

    const VERKEY_BASE58: &str = "6MkfJTyeCL8MGvZntdpXzpitL6bM6uwCFz8xcYar18xQBh7";

    const DID_PEER: &str = "did:peer:2.Vz6MkfJTyeCL8MGvZntdpXzpitL6bM6uwCFz8xcYar18xQBh7.SeyJwcmlvcml0eSI6MCwiciI6WyI4UHMyV29zSjlBVjFlWFBvSktzRUpkTTNOY2hQaFN5UzhxRnQ2TFFVVEt2MiIsIkhlemNlMlVXTVozd1VoVmtoMkxmS1NzOG5Eeld3enMyV2luN0V6Tk4zWWFSIl0sInJlY2lwaWVudEtleXMiOlsiMlpIRkZoekEyWHRURDZoSnF6TDd1eCMxIl0sInMiOiJodHRwOi8vbG9jYWxob3N0OjgwODAvYWdlbmN5L21zZyIsInQiOiJJbmR5QWdlbnQifQ";

    #[test]
    fn test_deserialization_legacy() {
        let did_doc: DidDocument =
            deserialize_legacy_or_new_diddoc_str(LEGACY_DID_DOC_JSON.into()).unwrap();
        assert_eq!(did_doc.id().to_string(), DID_PEER);
        assert_eq!(did_doc.verification_method().len(), 1);
        assert_eq!(did_doc.authentication().len(), 0);
        assert_eq!(did_doc.assertion_method().len(), 0);
        assert_eq!(did_doc.key_agreement().len(), 0);
        assert_eq!(did_doc.service().len(), 1);

        let verification_method = did_doc.verification_method().first().unwrap();
        assert_eq!(verification_method.id().to_string(), "#1");
        assert_eq!(verification_method.controller().to_string(), DID_PEER);
        assert_eq!(
            verification_method
                .public_key()
                .unwrap()
                .prefixless_fingerprint(),
            VERKEY_BASE58
        );

        let service = did_doc.service().first().unwrap();
        assert_eq!(
            service.id().to_string(),
            "did:example:123456789abcdefghi;indy"
        );
        assert_eq!(
            service.service_endpoint().to_string().as_str(),
            "http://localhost:8080/agency/msg"
        );

        let recipient_key = match service
            .extra_field_recipient_keys()
            .unwrap()
            .first()
            .unwrap()
        {
            ServiceKeyKind::Reference(did_url) => did_doc
                .dereference_key(did_url)
                .unwrap()
                .public_key()
                .unwrap()
                .prefixless_fingerprint(),
            _ => panic!("Expected reference"),
        };
        assert_eq!(recipient_key, VERKEY_BASE58);
        assert_eq!(service.extra_field_priority().unwrap(), 0);
        assert_eq!(service.extra_field_routing_keys().unwrap().len(), 2);
    }
}
