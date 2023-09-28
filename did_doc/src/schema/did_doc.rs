use std::collections::HashMap;

use did_parser::{Did, DidUrl};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    service::Service,
    types::uri::Uri,
    utils::OneOrList,
    verification_method::{VerificationMethod, VerificationMethodKind},
};
use crate::error::DidDocumentBuilderError;

pub type ControllerAlias = OneOrList<Did>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct DidDocument<E> {
    id: Did,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    also_known_as: Vec<Uri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    controller: Option<ControllerAlias>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    verification_method: Vec<VerificationMethod>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    authentication: Vec<VerificationMethodKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    assertion_method: Vec<VerificationMethodKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    key_agreement: Vec<VerificationMethodKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    capability_invocation: Vec<VerificationMethodKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    capability_delegation: Vec<VerificationMethodKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    service: Vec<Service<E>>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

impl<E> DidDocument<E> {
    pub fn builder(id: Did) -> DidDocumentBuilder<E> {
        DidDocumentBuilder::new(id)
    }

    pub fn id(&self) -> &Did {
        &self.id
    }

    pub fn also_known_as(&self) -> &[Uri] {
        self.also_known_as.as_ref()
    }

    pub fn controller(&self) -> Option<&OneOrList<Did>> {
        self.controller.as_ref()
    }

    pub fn verification_method(&self) -> &[VerificationMethod] {
        self.verification_method.as_ref()
    }

    pub fn authentication(&self) -> &[VerificationMethodKind] {
        self.authentication.as_ref()
    }

    pub fn assertion_method(&self) -> &[VerificationMethodKind] {
        self.assertion_method.as_ref()
    }

    pub fn key_agreement(&self) -> &[VerificationMethodKind] {
        self.key_agreement.as_ref()
    }

    pub fn capability_invocation(&self) -> &[VerificationMethodKind] {
        self.capability_invocation.as_ref()
    }

    pub fn capability_delegation(&self) -> &[VerificationMethodKind] {
        self.capability_delegation.as_ref()
    }

    pub fn service(&self) -> &[Service<E>] {
        self.service.as_ref()
    }

    pub fn extra_field(&self, key: &str) -> Option<&Value> {
        self.extra.get(key)
    }

    pub fn dereference_key(&self, reference: &DidUrl) -> Option<&VerificationMethod> {
        // TODO: Should check controller (if present)
        self.verification_method
            .iter()
            .find(|vm| vm.id().fragment() == reference.fragment())
    }

    pub fn validate(&self) -> Result<(), DidDocumentBuilderError> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct DidDocumentBuilder<E> {
    id: Did,
    also_known_as: Vec<Uri>,
    controller: Vec<Did>,
    verification_method: Vec<VerificationMethod>,
    authentication: Vec<VerificationMethodKind>,
    assertion_method: Vec<VerificationMethodKind>,
    key_agreement: Vec<VerificationMethodKind>,
    capability_invocation: Vec<VerificationMethodKind>,
    capability_delegation: Vec<VerificationMethodKind>,
    service: Vec<Service<E>>,
    extra: HashMap<String, Value>,
}

impl<E> Default for DidDocumentBuilder<E> {
    fn default() -> Self {
        Self {
            id: Default::default(),
            also_known_as: Default::default(),
            controller: Default::default(),
            verification_method: Default::default(),
            authentication: Default::default(),
            assertion_method: Default::default(),
            key_agreement: Default::default(),
            capability_invocation: Default::default(),
            capability_delegation: Default::default(),
            service: Default::default(),
            extra: Default::default(),
        }
    }
}

impl<E> DidDocumentBuilder<E> {
    pub fn new(id: Did) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn add_also_known_as(mut self, also_known_as: Uri) -> Self {
        self.also_known_as.push(also_known_as);
        self
    }

    pub fn add_controller(mut self, controller: Did) -> Self {
        self.controller.push(controller);
        self
    }

    pub fn add_verification_method(mut self, verification_method: VerificationMethod) -> Self {
        self.verification_method.push(verification_method);
        self
    }

    pub fn add_authentication_method(mut self, method: VerificationMethod) -> Self {
        self.authentication
            .push(VerificationMethodKind::Resolved(method));
        self
    }

    pub fn add_authentication_reference(mut self, reference: DidUrl) -> Self {
        self.authentication
            .push(VerificationMethodKind::Resolvable(reference));
        self
    }

    pub fn add_assertion_method(mut self, method: VerificationMethod) -> Self {
        self.assertion_method
            .push(VerificationMethodKind::Resolved(method));
        self
    }

    pub fn add_assertion_method_reference(mut self, reference: DidUrl) -> Self {
        self.assertion_method
            .push(VerificationMethodKind::Resolvable(reference));
        self
    }

    pub fn add_key_agreement(mut self, key_agreement: VerificationMethod) -> Self {
        self.key_agreement
            .push(VerificationMethodKind::Resolved(key_agreement));
        self
    }

    pub fn add_key_agreement_reference(mut self, reference: DidUrl) -> Self {
        self.key_agreement
            .push(VerificationMethodKind::Resolvable(reference));
        self
    }

    pub fn add_capability_invocation(mut self, capability_invocation: VerificationMethod) -> Self {
        self.capability_invocation
            .push(VerificationMethodKind::Resolved(capability_invocation));
        self
    }

    pub fn add_capability_invocation_reference(mut self, reference: DidUrl) -> Self {
        self.capability_invocation
            .push(VerificationMethodKind::Resolvable(reference));
        self
    }

    pub fn add_capability_delegation(mut self, capability_delegation: VerificationMethod) -> Self {
        self.capability_delegation
            .push(VerificationMethodKind::Resolved(capability_delegation));
        self
    }

    pub fn add_capability_delegation_refrence(mut self, reference: DidUrl) -> Self {
        self.capability_delegation
            .push(VerificationMethodKind::Resolvable(reference));
        self
    }

    pub fn add_service(mut self, service: Service<E>) -> Self {
        self.service.push(service);
        self
    }

    pub fn add_extra_field(mut self, key: String, value: Value) -> Self {
        self.extra.insert(key, value);
        self
    }

    pub fn build(self) -> DidDocument<E> {
        let controller = if self.controller.is_empty() {
            None
        } else {
            Some(OneOrList::List(self.controller))
        };
        DidDocument {
            id: self.id,
            also_known_as: self.also_known_as,
            controller,
            verification_method: self.verification_method,
            authentication: self.authentication,
            assertion_method: self.assertion_method,
            key_agreement: self.key_agreement,
            capability_invocation: self.capability_invocation,
            capability_delegation: self.capability_delegation,
            service: self.service,
            extra: self.extra,
        }
    }
}

impl<E> From<DidDocument<E>> for DidDocumentBuilder<E> {
    fn from(did_document: DidDocument<E>) -> Self {
        let controller = match did_document.controller {
            Some(OneOrList::List(list)) => list,
            _ => Vec::new(),
        };

        Self {
            id: did_document.id,
            also_known_as: did_document.also_known_as,
            controller,
            verification_method: did_document.verification_method,
            authentication: did_document.authentication,
            assertion_method: did_document.assertion_method,
            key_agreement: did_document.key_agreement,
            capability_invocation: did_document.capability_invocation,
            capability_delegation: did_document.capability_delegation,
            service: did_document.service,
            extra: did_document.extra,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{service::ServiceBuilder, verification_method::VerificationMethodType};

    #[test]
    fn test_did_document_builder() {
        let id = Did::parse("did:example:123456789abcdefghi".to_string()).unwrap();
        let also_known_as = Uri::new("https://example.com").unwrap();
        let controller = Did::parse("did:example:controller".to_string()).unwrap();

        let vm1_id = DidUrl::parse("did:example:vm1#vm1".to_string()).unwrap();
        let verification_method = VerificationMethod::builder(
            vm1_id.clone(),
            Did::parse("did:example:vm1".to_string()).unwrap(),
            VerificationMethodType::Ed25519VerificationKey2018,
        )
        .add_public_key_base58("H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string())
        .build();
        let authentication_reference = DidUrl::parse("did:example:authref".to_string()).unwrap();
        let assertion_method = VerificationMethod::builder(
            DidUrl::parse("did:example:am1".to_string()).unwrap(),
            Did::parse("did:example:am2".to_string()).unwrap(),
            VerificationMethodType::Ed25519VerificationKey2018,
        )
        .add_public_key_base58("H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string())
        .build();

        let service_id = Uri::new("did:example:123456789abcdefghi;service-1").unwrap();
        let service_type = "test-service".to_string();
        let service_endpoint = "https://example.com/service";
        let service =
            ServiceBuilder::<()>::new(service_id, service_endpoint.try_into().unwrap(), ())
                .add_service_type(service_type)
                .unwrap()
                .build();

        let document = DidDocumentBuilder::new(id.clone())
            .add_also_known_as(also_known_as.clone())
            .add_controller(controller.clone())
            .add_verification_method(verification_method.clone())
            .add_authentication_method(verification_method.clone())
            .add_authentication_reference(authentication_reference.clone())
            .add_assertion_method(assertion_method.clone())
            .add_assertion_method_reference(authentication_reference.clone())
            .add_key_agreement(verification_method.clone())
            .add_key_agreement_reference(authentication_reference.clone())
            .add_capability_invocation(verification_method.clone())
            .add_capability_invocation_reference(authentication_reference.clone())
            .add_capability_delegation(verification_method.clone())
            .add_capability_delegation_refrence(authentication_reference.clone())
            .add_service(service.clone())
            .build();

        assert_eq!(document.id(), &id);
        assert_eq!(document.also_known_as(), &[also_known_as]);
        assert_eq!(
            document.controller(),
            Some(&OneOrList::List(vec![controller]))
        );
        assert_eq!(
            document.verification_method(),
            &[verification_method.clone()]
        );
        assert_eq!(
            document.authentication(),
            &[
                VerificationMethodKind::Resolved(verification_method.clone()),
                VerificationMethodKind::Resolvable(authentication_reference.clone())
            ]
        );
        assert_eq!(
            document.assertion_method(),
            &[
                VerificationMethodKind::Resolved(assertion_method),
                VerificationMethodKind::Resolvable(authentication_reference.clone())
            ]
        );
        assert_eq!(
            document.key_agreement(),
            &[
                VerificationMethodKind::Resolved(verification_method.clone()),
                VerificationMethodKind::Resolvable(authentication_reference.clone())
            ]
        );
        assert_eq!(
            document.capability_invocation(),
            &[
                VerificationMethodKind::Resolved(verification_method.clone()),
                VerificationMethodKind::Resolvable(authentication_reference.clone())
            ]
        );
        assert_eq!(
            document.capability_delegation(),
            &[
                VerificationMethodKind::Resolved(verification_method),
                VerificationMethodKind::Resolvable(authentication_reference)
            ]
        );
        assert_eq!(document.service(), &[service]);

        let vm = document.dereference_key(&vm1_id);
        if let Some(vm) = vm {
            assert_eq!(vm.id(), &vm1_id);
        } else {
            panic!("Verification method not found")
        };
    }
}
