use url::Url;

use service_aries::AriesService;

use crate::did_doc::model::{
    Authentication, DdoKeyReference, Ed25519PublicKey, CONTEXT, KEY_AUTHENTICATION_TYPE, KEY_TYPE,
};
use crate::utils::validation::validate_verkey;
use crate::error::{MessagesError, MesssagesErrorKind, MessagesResult};

pub mod model;
pub mod service_aries_public;
pub mod service_aries;
pub mod service_resolvable;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DidDoc {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    #[serde(rename = "publicKey")] // todo: remove this, use authentication
    pub public_key: Vec<Ed25519PublicKey>,
    #[serde(default)]
    pub authentication: Vec<Authentication>,
    pub service: Vec<AriesService>,
}

impl Default for DidDoc {
    fn default() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: String::new(),
            public_key: vec![],
            authentication: vec![],
            service: vec![AriesService::default()],
        }
    }
}

impl DidDoc {
    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }

    pub fn set_service_endpoint(&mut self, service_endpoint: String) {
        self.service.get_mut(0).map(|service| {
            service.service_endpoint = service_endpoint;
            service
        });
    }

    pub fn set_recipient_keys(&mut self, recipient_keys: Vec<String>) {
        let mut key_id = 0;

        recipient_keys.iter().for_each(|key_in_base58| {
            key_id += 1;

            let key_reference = DidDoc::build_key_reference(&self.id, &key_id.to_string());

            self.public_key.push(Ed25519PublicKey {
                id: key_reference.clone(),
                type_: String::from(KEY_TYPE),
                controller: self.id.clone(),
                public_key_base_58: key_in_base58.clone(),
            });

            self.authentication.push(Authentication {
                type_: String::from(KEY_AUTHENTICATION_TYPE),
                public_key: key_reference.clone(),
            });

            self.service.get_mut(0).map(|service| {
                service.recipient_keys.push(key_in_base58.clone());
                service
            });
        });
    }

    pub fn set_routing_keys(&mut self, routing_keys: Vec<String>) {
        routing_keys.iter().for_each(|key| {
            // Note: comment lines 123 - 134 and append key instead key_reference to be compatible with Streetcred
            //                id += 1;
            //
            //                let key_id = id.to_string();
            //                let key_reference = DidDoc::_build_key_reference(&self.id, &key_id);

            // self.public_key.push(
            //     Ed25519PublicKey {
            //         id: key_id,
            //         type_: String::from(KEY_TYPE),
            //         controller: self.id.clone(),
            //         public_key_base_58: key.clone(),
            //     });

            self.service.get_mut(0).map(|service| {
                service.routing_keys.push(key.to_string());
                service
            });
        });
    }

    pub fn validate(&self) -> MessagesResult<()> {
        if self.context != CONTEXT {
            return Err(MessagesError::from_msg(
                MesssagesErrorKind::InvalidJson,
                format!(
                    "DIDDoc validation failed: Unsupported @context value: {:?}",
                    self.context
                ),
            ));
        }

        if self.id.is_empty() {
            return Err(MessagesError::from_msg(
                MesssagesErrorKind::InvalidJson,
                "DIDDoc validation failed: id is empty",
            ));
        }

        for service in self.service.iter() {
            Url::parse(&service.service_endpoint).map_err(|err| {
                MessagesError::from_msg(
                    MesssagesErrorKind::InvalidJson,
                    format!(
                        "DIDDoc validation failed: Invalid endpoint \"{:?}\", err: {:?}",
                        service.service_endpoint, err
                    ),
                )
            })?;

            service.recipient_keys.iter().try_for_each(|recipient_key_entry| {
                let public_key = self.find_key(recipient_key_entry)?;
                self.is_authentication_key(&public_key.id)?;
                Ok::<_, MessagesError>(())
            })?;

            service.routing_keys.iter().try_for_each(|routing_key_entry| {
                // todo: use same approach as for recipient keys above, but for that we need to
                // first update implementation of set_routing_keys() to include routing keys in
                // 'authentication' verification method of the DDO
                // That represents assumption that 'routing_key_entry' is always key value and not key reference
                validate_verkey(routing_key_entry)?;
                Ok::<_, MessagesError>(())
            })?;
        }

        Ok(())
    }

    pub fn recipient_keys(&self) -> Vec<String> {
        let service: AriesService = match self.service.get(0).cloned() {
            Some(service) => service,
            None => return Vec::new(),
        };
        let recipient_keys = service
            .recipient_keys
            .iter()
            .filter_map(|recipient_key_entry| match self.find_key(recipient_key_entry) {
                Ok(key) => Some(key.public_key_base_58),
                Err(err) => {
                    warn!(
                        "ddo:: recipient_keys >> error occurred trying to recipient key entry {}, error: {}",
                        recipient_key_entry, err
                    );
                    None
                }
            })
            .collect();
        recipient_keys
    }

    pub fn routing_keys(&self) -> Vec<String> {
        let service: AriesService = match self.service.get(0).cloned() {
            Some(service) => service,
            None => return Vec::new(),
        };
        service.routing_keys.iter().cloned().collect()
    }

    pub fn get_endpoint(&self) -> String {
        match self.service.get(0) {
            Some(service) => service.service_endpoint.to_string(),
            None => String::new(),
        }
    }

    pub fn get_service(&self) -> MessagesResult<AriesService> {
        let service: &AriesService = self.service.get(0).ok_or(MessagesError::from_msg(
            MesssagesErrorKind::InvalidState,
            format!("No service found on did doc: {:?}", self),
        ))?;
        let recipient_keys = self.recipient_keys();
        let routing_keys = self.routing_keys();
        Ok(AriesService {
            recipient_keys,
            routing_keys,
            ..service.clone()
        })
    }

    fn find_key(&self, key_value_or_reference: &str) -> MessagesResult<Ed25519PublicKey> {
        let public_key = match validate_verkey(key_value_or_reference) {
            Ok(key) => self.find_key_by_value(key),
            Err(_) => {
                let key_ref = DidDoc::parse_key_reference(key_value_or_reference)?;
                self.find_key_by_reference(&key_ref)
            }
        }?;
        Self::_validate_ed25519_key(&public_key)?;
        Ok(public_key)
    }

    fn _validate_ed25519_key(public_key: &Ed25519PublicKey) -> MessagesResult<()> {
        if public_key.type_ != KEY_TYPE {
            return Err(MessagesError::from_msg(
                MesssagesErrorKind::InvalidJson,
                format!(
                    "DIDDoc validation failed: Unsupported PublicKey type: {:?}",
                    public_key.type_
                ),
            ));
        }
        validate_verkey(&public_key.public_key_base_58)?;
        Ok(())
    }

    fn find_key_by_reference(&self, key_ref: &DdoKeyReference) -> MessagesResult<Ed25519PublicKey> {
        let public_key = self
            .public_key
            .iter()
            .find(|ddo_keys| match &key_ref.did {
                None => ddo_keys.id == key_ref.key_id,
                Some(did) => ddo_keys.id == key_ref.key_id || ddo_keys.id == format!("{}#{}", did, key_ref.key_id),
            })
            .ok_or(MessagesError::from_msg(
                MesssagesErrorKind::InvalidJson,
                format!("Failed to find entry in public_key by key reference: {:?}", key_ref),
            ))?;
        Ok(public_key.clone())
    }

    fn find_key_by_value(&self, key: String) -> MessagesResult<Ed25519PublicKey> {
        let public_key = self
            .public_key
            .iter()
            .find(|ddo_keys| ddo_keys.public_key_base_58 == key)
            .ok_or(MessagesError::from_msg(
                MesssagesErrorKind::InvalidJson,
                format!("Failed to find entry in public_key by key value: {}", key),
            ))?;
        Ok(public_key.clone())
    }

    fn is_authentication_key(&self, key: &str) -> MessagesResult<()> {
        if self.authentication.is_empty() {
            // todo: remove this, was probably to support legacy implementations
            return Ok(());
        }
        let authentication_key = self
            .authentication
            .iter()
            .find(|auth_key| {
                if auth_key.public_key == key {
                    return true;
                }
                match DidDoc::parse_key_reference(&auth_key.public_key) {
                    Ok(auth_public_key_ref) => auth_public_key_ref.key_id == key,
                    Err(_) => false,
                }
            })
            .ok_or(MessagesError::from_msg(
                MesssagesErrorKind::InvalidJson,
                format!(
                    "DIDDoc validation failed: Cannot find Authentication record key: {:?}",
                    key
                ),
            ))?;

        if authentication_key.type_ != KEY_AUTHENTICATION_TYPE && authentication_key.type_ != KEY_TYPE {
            return Err(MessagesError::from_msg(
                MesssagesErrorKind::InvalidJson,
                format!(
                    "DIDDoc validation failed: Unsupported Authentication type: {:?}",
                    authentication_key.type_
                ),
            ));
        }

        Ok(())
    }

    fn build_key_reference(did: &str, id: &str) -> String {
        format!("{}#{}", did, id)
    }

    fn key_parts(key: &str) -> Vec<&str> {
        key.split('#').collect()
    }

    fn parse_key_reference(key_reference: &str) -> MessagesResult<DdoKeyReference> {
        let pars: Vec<&str> = DidDoc::key_parts(key_reference);
        match pars.len() {
            0 => Err(MessagesError::from_msg(
                MesssagesErrorKind::InvalidJson,
                format!("DIDDoc validation failed: Invalid key reference: {:?}", key_reference),
            )),
            1 => Ok(DdoKeyReference {
                did: None,
                key_id: pars[0].to_string(),
            }),
            _ => Ok(DdoKeyReference {
                did: Some(pars[0].to_string()),
                key_id: pars[1].to_string(),
            }),
        }
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use crate::did_doc::model::*;
    use crate::did_doc::service_aries::AriesService;
    use crate::did_doc::DidDoc;

    pub fn _key_1() -> String {
        String::from("GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL")
    }

    pub fn _key_2() -> String {
        String::from("Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR")
    }

    pub fn _key_3() -> String {
        String::from("3LYuxJBJkngDbvJj4zjx13DBUdZ2P96eNybwd2n9L9AU")
    }

    pub fn _did() -> String {
        String::from("VsKV7grR1BUE29mG2Fm2kX")
    }

    pub fn _service_endpoint() -> String {
        String::from("http://localhost:8080")
    }

    pub fn _recipient_keys() -> Vec<String> {
        vec![_key_1()]
    }

    pub fn _routing_keys() -> Vec<String> {
        vec![_key_2(), _key_3()]
    }

    pub fn _routing_keys_1() -> Vec<String> {
        vec![_key_1(), _key_3()]
    }

    pub fn _key_reference_1() -> String {
        DidDoc::build_key_reference(&_did(), "1")
    }

    pub fn _key_reference_full_1_typed() -> DdoKeyReference {
        DdoKeyReference {
            did: Some(_did()),
            key_id: "1".to_string(),
        }
    }

    pub fn _key_reference_2() -> String {
        DidDoc::build_key_reference(&_did(), "2")
    }

    pub fn _key_reference_3() -> String {
        DidDoc::build_key_reference(&_did(), "3")
    }

    pub fn _label() -> String {
        String::from("test")
    }

    pub fn _did_doc_vcx_legacy() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _did(),
            public_key: vec![Ed25519PublicKey {
                id: "1".to_string(),
                type_: KEY_TYPE.to_string(),
                controller: _did(),
                public_key_base_58: _key_1(),
            }],
            authentication: vec![Authentication {
                type_: KEY_AUTHENTICATION_TYPE.to_string(),
                public_key: _key_reference_1(),
            }],
            service: vec![AriesService {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_reference_1()],
                routing_keys: vec![_key_2(), _key_3()],
                ..Default::default()
            }],
        }
    }

    pub fn _did_doc_inlined_recipient_keys() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _did(),
            public_key: vec![Ed25519PublicKey {
                id: _key_reference_1(),
                type_: KEY_TYPE.to_string(),
                controller: _did(),
                public_key_base_58: _key_1(),
            }],
            authentication: vec![Authentication {
                type_: KEY_AUTHENTICATION_TYPE.to_string(),
                public_key: _key_reference_1(),
            }],
            service: vec![AriesService {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_1()],
                routing_keys: vec![_key_2(), _key_3()],
                ..Default::default()
            }],
        }
    }

    pub fn _did_doc_recipient_keys_by_value() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _did(),
            public_key: vec![
                Ed25519PublicKey {
                    id: _key_reference_1(),
                    type_: KEY_TYPE.to_string(),
                    controller: _did(),
                    public_key_base_58: _key_1(),
                },
                Ed25519PublicKey {
                    id: _key_reference_2(),
                    type_: KEY_TYPE.to_string(),
                    controller: _did(),
                    public_key_base_58: _key_2(),
                },
                Ed25519PublicKey {
                    id: _key_reference_3(),
                    type_: KEY_TYPE.to_string(),
                    controller: _did(),
                    public_key_base_58: _key_3(),
                },
            ],
            authentication: vec![Authentication {
                type_: KEY_AUTHENTICATION_TYPE.to_string(),
                public_key: _key_reference_1(),
            }],
            service: vec![AriesService {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_1()],
                routing_keys: vec![_key_2(), _key_3()],
                ..Default::default()
            }],
        }
    }

    pub fn _did_doc_empty_routing() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _did(),
            public_key: vec![Ed25519PublicKey {
                id: _key_1(),
                type_: KEY_TYPE.to_string(),
                controller: _did(),
                public_key_base_58: _key_1(),
            }],
            authentication: vec![Authentication {
                type_: KEY_AUTHENTICATION_TYPE.to_string(),
                public_key: _key_1(),
            }],
            service: vec![AriesService {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_1()],
                routing_keys: vec![],
                ..Default::default()
            }],
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use crate::did_doc::test_utils::*;
    use crate::did_doc::DidDoc;
    use crate::utils::devsetup::SetupEmpty;

    #[test]
    fn test_did_doc_build_works() {
        let mut did_doc: DidDoc = DidDoc::default();
        did_doc.set_id(_did());
        did_doc.set_service_endpoint(_service_endpoint());
        did_doc.set_recipient_keys(_recipient_keys());
        did_doc.set_routing_keys(_routing_keys());

        assert_eq!(_did_doc_inlined_recipient_keys(), did_doc);
    }

    #[test]
    fn test_did_doc_validate_works() {
        _did_doc_vcx_legacy().validate().unwrap();
        _did_doc_inlined_recipient_keys().validate().unwrap();
        _did_doc_recipient_keys_by_value().validate().unwrap();
        _did_doc_empty_routing().validate().unwrap();
    }

    #[test]
    fn test_did_doc_key_for_reference_works() {
        let ddo = _did_doc_vcx_legacy();
        let key_resolved = ddo.find_key_by_reference(&_key_reference_full_1_typed()).unwrap();
        assert_eq!(_key_1(), key_resolved.public_key_base_58);
    }

    #[test]
    fn test_did_doc_resolve_recipient_key_by_reference_works() {
        let ddo: DidDoc = serde_json::from_value(json!({
            "@context": "https://w3id.org/did/v1",
            "id": "testid",
            "publicKey": [
                {
                    "id": "testid#1",
                    "type": "Ed25519VerificationKey2018",
                    "controller": "testid",
                    "publicKeyBase58": "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL"
                }
            ],
            "authentication": [
                {
                    "type": "Ed25519SignatureAuthentication2018",
                    "publicKey": "testid#1"
                }
            ],
            "service": [
                {
                    "id": "did:example:123456789abcdefghi;indy",
                    "type": "IndyAgent",
                    "priority": 0,
                    "recipientKeys": [
                        "testid#1"
                    ],
                    "routingKeys": [
                        "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
                        "3LYuxJBJkngDbvJj4zjx13DBUdZ2P96eNybwd2n9L9AU"
                    ],
                    "serviceEndpoint": "http://localhost:8080"
                }
            ]
        }))
        .unwrap();
        assert_eq!(_recipient_keys(), ddo.recipient_keys());
    }

    #[test]
    fn test_did_doc_resolve_recipient_keys_works() {
        let recipient_keys = _did_doc_vcx_legacy().recipient_keys();
        assert_eq!(_recipient_keys(), recipient_keys);

        let recipient_keys = _did_doc_recipient_keys_by_value().recipient_keys();
        assert_eq!(_recipient_keys(), recipient_keys);
    }

    #[test]
    fn test_did_doc_resolve_routing_keys_works() {
        let routing_keys = _did_doc_vcx_legacy().routing_keys();
        assert_eq!(_routing_keys(), routing_keys);

        let routing_keys = _did_doc_recipient_keys_by_value().routing_keys();
        assert_eq!(_routing_keys(), routing_keys);
    }

    #[test]
    fn test_did_doc_serialization() {
        SetupEmpty::init();
        let ddo = _did_doc_vcx_legacy();
        let ddo_value = serde_json::to_value(&ddo).unwrap();
        let expected_value = json!({
            "@context": "https://w3id.org/did/v1",
            "id": "VsKV7grR1BUE29mG2Fm2kX",
            "publicKey": [
                {
                    "id": "1",
                    "type": "Ed25519VerificationKey2018",
                    "controller": "VsKV7grR1BUE29mG2Fm2kX",
                    "publicKeyBase58": "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL"
                }
            ],
            "authentication": [
                {
                    "type": "Ed25519SignatureAuthentication2018",
                    "publicKey": "VsKV7grR1BUE29mG2Fm2kX#1"
                }
            ],
            "service": [
                {
                    "id": "did:example:123456789abcdefghi;indy",
                    "type": "IndyAgent",
                    "priority": 0,
                    "recipientKeys": [
                        "VsKV7grR1BUE29mG2Fm2kX#1"
                    ],
                    "routingKeys": [
                        "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
                        "3LYuxJBJkngDbvJj4zjx13DBUdZ2P96eNybwd2n9L9AU"
                    ],
                    "serviceEndpoint": "http://localhost:8080"
                }
            ]
        });
        assert_eq!(expected_value, ddo_value);
    }

    #[test]
    fn test_did_doc_build_key_reference_works() {
        assert_eq!(_key_reference_1(), DidDoc::build_key_reference(&_did(), "1"));
    }

    #[test]
    fn test_did_doc_parse_key_reference_works() {
        assert_eq!(
            _key_reference_full_1_typed(),
            DidDoc::parse_key_reference(&_key_reference_1()).unwrap()
        );
    }
}
