use url::Url;
use crate::did_doc::model::{Authentication, CONTEXT, Ed25519PublicKey, KEY_AUTHENTICATION_TYPE, KEY_TYPE};
use crate::error::{VcxError, VcxErrorKind, VcxResult};
use service_aries::AriesService;
use crate::utils::validation::validate_verkey;
pub mod model;
pub mod service_aries;

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
        self.service.get_mut(0)
            .map(|service| {
                service.service_endpoint = service_endpoint;
                service
            });
    }

    pub fn set_recipient_keys(&mut self, recipient_keys: Vec<String>) {
        let mut id = 0;

        recipient_keys
            .iter()
            .for_each(|key| {
                id += 1;

                let key_id = id.to_string();
                let key_reference = DidDoc::build_key_reference(&self.id, &key_id);

                self.public_key.push(
                    Ed25519PublicKey {
                        id: key_id,
                        type_: String::from(KEY_TYPE),
                        controller: self.id.clone(),
                        public_key_base_58: key.clone(),
                    });

                self.authentication.push(
                    Authentication {
                        type_: String::from(KEY_AUTHENTICATION_TYPE),
                        public_key: key_reference.clone(),
                    });


                self.service.get_mut(0)
                    .map(|service| {
                        service.recipient_keys.push(key_reference);
                        service
                    });
            });
    }

    pub fn set_routing_keys(&mut self, routing_keys: Vec<String>) {
        routing_keys
            .iter()
            .for_each(|key| {
                // Note: comment lines 123 - 134 and append key instead key_reference to be compatible with Streetcred
//                id += 1;
//
//                let key_id = id.to_string();
//                let key_reference = DidDoc::_build_key_reference(&self.id, &key_id);
//
//                self.public_key.push(
//                    Ed25519PublicKey {
//                        id: key_id,
//                        type_: String::from(KEY_TYPE),
//                        controller: self.id.clone(),
//                        public_key_base_58: key.clone(),
//                    });

                self.service.get_mut(0)
                    .map(|service| {
                        service.routing_keys.push(key.to_string());
                        service
                    });
            });
    }

    pub fn validate(&self) -> VcxResult<()> {
        if self.context != CONTEXT {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("DIDDoc validation failed: Unsupported @context value: {:?}", self.context)));
        }

//        if self.id.is_empty() {
//            return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, "DIDDoc validation failed: id is empty"));
//        }

        for service in self.service.iter() {
            Url::parse(&service.service_endpoint)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("DIDDoc validation failed: Invalid endpoint \"{:?}\", err: {:?}", service.service_endpoint, err)))?;

            service.recipient_keys
                .iter().try_for_each(|key| self.validate_recipient_key(key))?;

            service.routing_keys
                .iter().try_for_each(|key| self.validate_routing_key(key))?;
        }

        Ok(())
    }

    // TODO: Expects one service only
    pub fn resolve_keys(&self) -> (Vec<String>, Vec<String>) {
        let service: AriesService = match self.service.get(0).cloned() {
            Some(service) => service,
            None => return (Vec::new(), Vec::new())
        };

        let recipient_keys: Vec<String> =
            service.recipient_keys
                .iter()
                .map(|key| self.key_for_reference(key))
                .collect();

        let routing_keys: Vec<String> =
            service.routing_keys
                .iter()
                .map(|key| self.key_for_reference(key))
                .collect();

        (recipient_keys, routing_keys)
    }

    pub fn recipient_keys(&self) -> Vec<String> {
        let (recipient_keys, _) = self.resolve_keys();
        recipient_keys
    }

    pub fn routing_keys(&self) -> Vec<String> {
        let (_, routing_keys) = self.resolve_keys();
        routing_keys
    }

    pub fn get_endpoint(&self) -> String {
        match self.service.get(0) {
            Some(service) => service.service_endpoint.to_string(),
            None => String::new()
        }
    }

    // TODO: Expects one service only
    pub fn resolve_service(&self) -> VcxResult<AriesService> {
        let service: &AriesService = self.service.get(0).ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, format!("No service found on did doc: {:?}", self)))?;
        let (recipient_keys, routing_keys) = self.resolve_keys();
        Ok(AriesService {
            recipient_keys,
            routing_keys,
            ..service.clone()
        })
    }

    fn validate_recipient_key(&self, key: &str) -> VcxResult<()> {
        let public_key = self.validate_public_key(key)?;
        self.validate_authentication(&public_key.id)
    }

    fn validate_routing_key(&self, key: &str) -> VcxResult<()> {
        if DidDoc::key_parts(key).len() == 2 {
            self.validate_public_key(key)?;
        } else {
            validate_verkey(key)?;
        }
        Ok(())
    }

    fn validate_public_key(&self, target_key: &str) -> VcxResult<&Ed25519PublicKey> {
        let id = DidDoc::parse_key_reference(target_key);

        let key = self.public_key.iter().find(|key_| key_.id == id || key_.public_key_base_58 == id)
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("DIDDoc validation failed: Cannot find PublicKey definition for key: {:?}", id)))?;

        if key.type_ != KEY_TYPE {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("DIDDoc validation failed: Unsupported PublicKey type: {:?}", key.type_)));
        }

        validate_verkey(&key.public_key_base_58)?;

        Ok(key)
    }

    fn validate_authentication(&self, target_key: &str) -> VcxResult<()> {
        if self.authentication.is_empty() {
            return Ok(());
        }

        let key = self.authentication.iter().find(|key_|
            key_.public_key == *target_key ||
                DidDoc::parse_key_reference(&key_.public_key) == *target_key)
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("DIDDoc validation failed: Cannot find Authentication section for key: {:?}", target_key)))?;

        if key.type_ != KEY_AUTHENTICATION_TYPE && key.type_ != KEY_TYPE {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("DIDDoc validation failed: Unsupported Authentication type: {:?}", key.type_)));
        }

        Ok(())
    }

    fn key_for_reference(&self, key_reference: &str) -> String {
        let id = DidDoc::parse_key_reference(key_reference);

        self.public_key.iter()
            .find(|key_| key_.id == id || key_.public_key_base_58 == id)
            .map(|key| key.public_key_base_58.clone())
            .unwrap_or(id)
    }

    fn build_key_reference(did: &str, id: &str) -> String {
        format!("{}#{}", did, id)
    }

    fn key_parts(key: &str) -> Vec<&str> {
        key.split('#').collect()
    }

    fn parse_key_reference(key_reference: &str) -> String {
        let pars: Vec<&str> = DidDoc::key_parts(key_reference);
        pars.get(1).or(pars.get(0)).map(|s| s.to_string()).unwrap_or_default()
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use crate::did_doc::DidDoc;
    use crate::did_doc::model::*;
    use crate::did_doc::service_aries::AriesService;

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

    pub fn _key_reference_2() -> String {
        DidDoc::build_key_reference(&_did(), "2")
    }

    pub fn _key_reference_3() -> String {
        DidDoc::build_key_reference(&_did(), "3")
    }

    pub fn _label() -> String {
        String::from("test")
    }

    pub fn _did_doc() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _did(),
            public_key: vec![
                Ed25519PublicKey { id: "1".to_string(), type_: KEY_TYPE.to_string(), controller: _did(), public_key_base_58: _key_1() },
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_reference_1() }
            ],
            service: vec![AriesService {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_reference_1()],
                routing_keys: vec![_key_2(), _key_3()],
                ..Default::default()
            }],
        }
    }

    pub fn _did_doc_2() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _did(),
            public_key: vec![
                Ed25519PublicKey { id: _key_reference_1(), type_: KEY_TYPE.to_string(), controller: _did(), public_key_base_58: _key_1() },
                Ed25519PublicKey { id: _key_reference_2(), type_: KEY_TYPE.to_string(), controller: _did(), public_key_base_58: _key_2() },
                Ed25519PublicKey { id: _key_reference_3(), type_: KEY_TYPE.to_string(), controller: _did(), public_key_base_58: _key_3() },
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_reference_1() }
            ],
            service: vec![AriesService {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_1()],
                routing_keys: vec![_key_2(), _key_3()],
                ..Default::default()
            }],
        }
    }

    pub fn _did_doc_3() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _did(),
            public_key: vec![
                Ed25519PublicKey { id: _key_1(), type_: KEY_TYPE.to_string(), controller: _did(), public_key_base_58: _key_1() },
                Ed25519PublicKey { id: _key_1(), type_: KEY_TYPE.to_string(), controller: _did(), public_key_base_58: _key_2() },
                Ed25519PublicKey { id: _key_1(), type_: KEY_TYPE.to_string(), controller: _did(), public_key_base_58: _key_3() },
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_1() }
            ],
            service: vec![AriesService {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_1()],
                routing_keys: vec![_key_2(), _key_3()],
                ..Default::default()
            }],
        }
    }

    pub fn _did_doc_4() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _did(),
            public_key: vec![
                Ed25519PublicKey { id: _key_1(), type_: KEY_TYPE.to_string(), controller: _did(), public_key_base_58: _key_1() },
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_1() }
            ],
            service: vec![AriesService {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_1()],
                routing_keys: vec![],
                ..Default::default()
            }],
        }
    }

    pub fn _did_doc_5() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _did(),
            public_key: vec![
                Ed25519PublicKey { id: _key_reference_1(), type_: KEY_TYPE.to_string(), controller: _did(), public_key_base_58: _key_1() },
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_reference_1() }
            ],
            service: vec![AriesService {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_1()],
                routing_keys: vec![_key_2(), _key_3()],
                ..Default::default()
            }],
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use crate::messages::a2a::MessageId;
    use crate::did_doc::test_utils::*;
    use crate::did_doc::DidDoc;
    use crate::messages::connection::invite::test_utils::_pairwise_invitation;
    use crate::utils::devsetup::SetupEmpty;

    use crate::did_doc::model::*;

    #[test]
    fn test_did_doc_build_works() {
        let mut did_doc: DidDoc = DidDoc::default();
        did_doc.set_id(_did());
        did_doc.set_service_endpoint(_service_endpoint());
        did_doc.set_recipient_keys(_recipient_keys());
        did_doc.set_routing_keys(_routing_keys());

        assert_eq!(_did_doc(), did_doc);
    }

    #[test]
    fn test_did_doc_validate_works() {
        _did_doc().validate().unwrap();
        _did_doc_2().validate().unwrap();
        _did_doc_3().validate().unwrap();
        _did_doc_4().validate().unwrap();
        _did_doc_5().validate().unwrap();
    }

    #[test]
    fn test_did_doc_key_for_reference_works() {
        assert_eq!(_key_1(), _did_doc().key_for_reference(&_key_reference_1()));
    }

    #[test]
    fn test_did_doc_resolve_keys_works() {
        let (recipient_keys, routing_keys) = _did_doc().resolve_keys();
        assert_eq!(_recipient_keys(), recipient_keys);
        assert_eq!(_routing_keys(), routing_keys);

        let (recipient_keys, routing_keys) = _did_doc_2().resolve_keys();
        assert_eq!(_recipient_keys(), recipient_keys);
        assert_eq!(_routing_keys(), routing_keys);
    }

    #[test]
    fn test_did_doc_serialization() {
        SetupEmpty::init();
        let ddo = _did_doc();
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
        assert_eq!(String::from("1"), DidDoc::parse_key_reference(&_key_reference_1()));
        assert_eq!(_key_1(), DidDoc::parse_key_reference(&_key_1()));
    }

    #[test]
    fn test_did_doc_from_invitation_works() {
        let mut did_doc = DidDoc::default();
        did_doc.set_id(MessageId::id().0);
        did_doc.set_service_endpoint(_service_endpoint());
        did_doc.set_recipient_keys(_recipient_keys());
        did_doc.set_routing_keys(_routing_keys());

        assert_eq!(did_doc, DidDoc::from(Invitation::Pairwise(_pairwise_invitation())));
    }
}
